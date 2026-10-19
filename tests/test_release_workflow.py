"""Regression tests for the single tested-revision release contract."""

from pathlib import Path
import re
import tomllib
import unittest


WORKFLOW = (
    Path(__file__).parents[1] / ".github" / "workflows" / "ci.yml"
).read_text(encoding="utf-8")


class ReleaseWorkflowTests(unittest.TestCase):
    """Keep publishing and validation attached to one immutable revision."""

    def test_build_hook_dependency_has_explicit_checkout_ownership(self) -> None:
        root = Path(__file__).parents[1]
        manifest = tomllib.loads((root / 'Cargo.toml').read_text())
        self.assertRegex(manifest['build-dependencies']['rhusky'], r'^2\.[0-9]+\.[0-9]+$')
        self.assertIn('install_from_build_script(env!("CARGO_MANIFEST_DIR"))',
                      (root / 'build.rs').read_text())
        self.assertNotIn('RHUSKY_REPOSITORY_ROOT', (root / '.cargo/config.toml').read_text())

    def test_compiler_pin_supports_the_current_hook_dependency(self) -> None:
        root = Path(__file__).parents[1]
        manifest = tomllib.loads((root / 'Cargo.toml').read_text())
        compiler = tomllib.loads((root / 'rust-toolchain.toml').read_text())['toolchain']['channel']
        self.assertEqual(compiler, manifest['workspace']['package']['rust-version'])
        self.assertGreaterEqual(tuple(map(int, compiler.split('.'))), (1, 97, 1))

    def test_dependency_merger_cannot_reuse_the_release_writer(self) -> None:
        root = Path(__file__).parents[1]
        merge = (root / '.github/workflows/dependabot-merge.yml').read_text()
        self.assertIn("if: github.event.workflow_run.conclusion == 'success'", merge)
        self.assertIn('secrets.RELEASE_MERGER_APP_ID', merge)
        self.assertIn('secrets.RELEASE_MERGER_APP_KEY', merge)
        self.assertIn('token: ${{ steps.merger.outputs.token }}', merge)
        self.assertNotIn('RELEASE_WRITER', merge)
        action = merge.split('uses: legra-ai/github-actions/.github/actions/merge-tested-dependabot@', 1)[1]
        self.assertIn('app-id: ${{ secrets.RELEASE_MERGER_APP_ID }}', action)

    def test_binding_publication_waits_for_verified_original_archives(self) -> None:
        for title, registry, package in [('Verify npm publication', 'npm', '@dotenvage/node'),
                                         ('Verify complete Python publication', 'pypi', 'dotenvage')]:
            block = WORKFLOW.split(f'      - name: {title}\n', 1)[1].split('\n      - ', 1)[0]
            self.assertRegex(block, r'uses: legra-ai/github-actions/\.github/actions/verify-registry-publication@[a-f0-9]{40}')
            for value in [f'registry: {registry}', f"package: '{package}'" if registry == 'npm' else f'package: {package}',
                          'version: ${{ needs.version-check.outputs.new_version }}',
                          'revision: ${{ needs.version-check.outputs.revision }}',
                          'archive: release-packages.zip']:
                self.assertIn(value, block)
        self.assertNotIn('name: Require published npm archive', WORKFLOW)

    def test_shared_release_policy_is_pinned_consistently(self) -> None:
        root = Path(__file__).parents[1]
        merge = (root / '.github/workflows/dependabot-merge.yml').read_text()
        title = (root / '.github/workflows/conventional-commits.yml').read_text()
        references = re.findall(r'uses: legra-ai/github-actions/[^@\s]+@(\S+)', WORKFLOW + merge + title)
        self.assertTrue(references)
        self.assertEqual(len(set(references)), 1)
        for reference in references:
            self.assertRegex(reference, r'^[a-f0-9]{40}$')

    def test_fork_pr_validation_does_not_require_app_secrets(self) -> None:
        prepare = WORKFLOW.split("  version-check:", 1)[1].split("  build-artifacts:", 1)[0]
        self.assertIn("if: github.event_name != 'pull_request'", prepare)
        self.assertIn("github.event_name == 'pull_request' && github.token", prepare)

    def test_required_gate_covers_all_platform_and_language_checks(self) -> None:
        self.assertIn("name: Validate Release", WORKFLOW)
        self.assertIn("needs: [fmt, clippy, test, test-npm, test-python, build-artifacts]", WORKFLOW)
        self.assertIn("if: always() && github.event_name == 'pull_request'", WORKFLOW)

    def test_artifact_builds_are_validated_before_merge(self) -> None:
        build = WORKFLOW.split("  build-artifacts:", 1)[1].split("  create-release:", 1)[0]
        self.assertIn("github.event_name == 'pull_request'", build)
        publish = WORKFLOW.split("  create-release:", 1)[1].split("  publish-all:", 1)[0]
        self.assertIn("if: needs.version-check.outputs.version_changed == 'true'", publish)

    def test_packaged_crate_is_validated(self) -> None:
        self.assertIn("python3 scripts/test-package.py", WORKFLOW)
        self.assertIn("cargo doc --locked --no-deps --all-features", WORKFLOW)

    def test_writers_use_scoped_app_tokens(self) -> None:
        root = Path(__file__).parents[1]
        merger = (root / ".github/workflows/dependabot-merge.yml").read_text()
        for source, identity in [(WORKFLOW, 'writer'), (merger, 'merger')]:
            self.assertIn("uses: actions/create-github-app-token@", source)
            self.assertIn("repositories: ${{ github.event.repository.name }}", source)
            self.assertIn(f"steps.{identity}.outputs.token", source)

    def test_existing_packages_require_verified_identity(self) -> None:
        self.assertNotIn("skip-existing: true", WORKFLOW)
        self.assertNotIn("EPUBLISHCONFLICT", WORKFLOW)
        for command in ["stage", "check-npm", "filter-python"]:
            self.assertIn(f"python3 scripts/release-packages.py {command}", WORKFLOW)

    def test_writer_action_version_is_consistent_across_jobs(self) -> None:
        root = Path(__file__).parents[1]
        merge = (root / ".github/workflows/dependabot-merge.yml").read_text()
        versions = re.findall(r"uses: actions/create-github-app-token@(\S+)", WORKFLOW + merge)
        self.assertTrue(versions)
        self.assertEqual(len(set(versions)), 1, versions)

    def test_release_creation_uses_the_authorized_writer(self) -> None:
        create = WORKFLOW.split("  create-release:\n", 1)[1].split("  publish-all:\n", 1)[0]
        self.assertIn("repositories: ${{ github.event.repository.name }}", create)
        self.assertLess(create.index("id: writer"), create.index("uses: actions/checkout@"))
        self.assertIn("token: ${{ steps.writer.outputs.token }}", create)
        upload = create.split("      - name: Create GitHub Release\n", 1)[1]
        self.assertIn("token: ${{ steps.writer.outputs.token }}", upload)
        self.assertNotIn("secrets.GITHUB_TOKEN", create)

    def test_release_recovery_and_completion_use_the_authorized_writer(self) -> None:
        publish = WORKFLOW.split("  publish-all:\n", 1)[1]
        self.assertIn("repositories: ${{ github.event.repository.name }}", publish)
        writer = publish.index("id: writer")
        recovery = publish.index("name: Preserve original package archives for recovery")
        self.assertLess(writer, recovery)
        self.assertIn("GH_TOKEN: ${{ steps.writer.outputs.token }}", publish)
        self.assertNotIn("GH_TOKEN: ${{ github.token }}", publish)

    def test_tag_and_release_use_the_prepared_revision(self) -> None:
        """The triggering SHA is not the generated and validated version commit."""
        self.assertNotIn("      - name: Push Release Tag\n", WORKFLOW)
        self.assertIn("legra-ai/github-actions/.github/actions/tag-tested-release@", WORKFLOW)
        self.assertNotIn("target_commitish: ${{ github.sha }}", WORKFLOW)
        self.assertIn("target_commitish: ${{ needs.version-check.outputs.revision }}", WORKFLOW)

    def test_old_parallel_publish_workflows_are_removed(self) -> None:
        workflows = Path(__file__).parents[1] / ".github/workflows"
        for name in ["auto-version-bump.yml", "dependabot-auto-merge.yml",
                     "dependabot-approve.yml", "dependabot-automerge.yml"]:
            self.assertFalse((workflows / name).exists(), name)

    def test_release_remains_draft_until_all_registries_finish(self) -> None:
        self.assertIn("draft: true", WORKFLOW)
        self.assertLess(WORKFLOW.index("name: Publish to PyPI"),
                        WORKFLOW.index("name: Complete release after all registries succeed"))

    def test_workflow_contract_tests_run_in_ci(self) -> None:
        self.assertIn("python3 -m unittest discover -s tests -p test_release_workflow.py", WORKFLOW)

    def test_ci_and_hooks_use_the_same_pinned_formatter(self) -> None:
        root = Path(__file__).parents[1]
        self.assertRegex((root / "rustfmt-toolchain").read_text().strip(),
                         r"^nightly-\d{4}-\d{2}-\d{2}$")
        self.assertIn("./scripts/fmt.sh --check", WORKFLOW)
        self.assertIn("./scripts/fmt.sh --check", (root / ".githooks/pre-commit").read_text())
        self.assertIn("toolchain: ${{ steps.formatter.outputs.toolchain }}", WORKFLOW)
        self.assertNotIn("cargo +nightly", WORKFLOW)


if __name__ == "__main__":
    unittest.main()
