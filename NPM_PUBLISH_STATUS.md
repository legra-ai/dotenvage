# NPM Publish Status and Next Steps

## Current Status

### ✅ Fixed Issues

1. **CI/CD Workflow** - Fixed workspace version parsing

   - Commit: `184db32` -
     `fix(ci): handle workspace version in version sync check`
   - The version sync check now correctly reads from
     `[workspace.package]`

2. **Version Sync Script** - Now updates all version references

   - Commit: `4064448` -
     `fix(scripts): sync napi cargo dependency version in version bump`
   - Updates `npm/package.json`
   - Updates `npm/dotenvage-napi/Cargo.toml` dependency version
   - Updates root `package.json` if present

3. **All Platform Binaries Built** ✅
   - Successfully built for all 5 platforms including:
     - ✅ `x86_64-unknown-linux-gnu` (Vercel needs this!)
     - ✅ `aarch64-unknown-linux-gnu`
     - ✅ `aarch64-apple-darwin`
     - ✅ `x86_64-pc-windows-msvc`
     - ✅ `aarch64-pc-windows-msvc`
   - All binaries downloaded and added to
     `/Users/jgeluk/Work/dotenvage/npm/`

### ❌ Remaining Issue

**npm Trusted Publisher Authentication Failure**

The GitHub Actions workflow fails at the npm publish step with:

```
npm error 404 Not Found - PUT https://registry.npmjs.org/@dotenvage%2fnode
npm notice Access token expired or revoked
```

## Root Cause

The npm Trusted Publisher (OIDC) is not properly configured on
npmjs.com, or the configuration doesn't match the workflow.

## Solution Options

### Option 1: Fix npm Trusted Publisher (Recommended)

**Steps**:

1. Go to https://www.npmjs.com/package/@dotenvage/node
2. Navigate to **Package settings** → **Trusted Publishers**
3. Check if a Trusted Publisher exists:
   - If yes, delete it and recreate
   - If no, create a new one
4. Configure with these **exact** values (case-sensitive):
   - **Owner**: `dataroadinc`
   - **Repository**: `dotenvage`
   - **Workflow filename**: `ci.yml`
   - **Environment**: (leave empty)
5. Save the configuration
6. Re-run the failed GitHub Actions workflow:
   ```bash
   cd /Users/jgeluk/Work/dotenvage
   gh run rerun 20928766281 --job=60134558807
   ```

### Option 2: Manual Publish with npm Token

**Steps**:

1. Login to npm:

   ```bash
   npm login
   ```

2. Restore prepublishOnly script:

   ```bash
   cd /Users/jgeluk/Work/dotenvage/npm
   # Edit package.json to restore: "prepublishOnly": "npm run build"
   ```

3. Publish manually:
   ```bash
   cd /Users/jgeluk/Work/dotenvage/npm
   npm publish --access public
   ```

**Note**: All binaries are already in the npm directory, so the
prepublishOnly build will just rebuild the current platform (macOS)
but all 7 platform binaries will be included in the package.

### Option 3: Use GitHub Actions with npm Token

**Steps**:

1. Create an npm automation token on npmjs.com
2. Add it as a GitHub secret: `NPM_TOKEN`
3. Update the workflow to use the token instead of OIDC:
   ```yaml
   - name: Publish to npmjs.org
     working-directory: npm
     run: pnpm publish --access public --no-git-checks
     env:
       NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
   ```

## Current Workaround

Business-composer is using a local link to dotenvage:

```json
"@dotenvage/node": "link:../dotenvage/npm"
```

This works for local development but **will fail on Vercel** because:

- Vercel can't access the local filesystem
- The link points to a local directory that doesn't exist on Vercel

## Next Steps

1. **Fix npm Trusted Publisher** (Option 1 above)
2. **Re-run the GitHub Actions workflow** to publish v0.1.9
3. **Update business-composer** to use `@dotenvage/node@^0.1.9`
4. **Commit and push** business-composer changes
5. **Vercel deployment will succeed** with the Linux x64 binary

## Files Modified

### Dotenvage

1. `.github/workflows/ci.yml` - Fixed workspace version parsing
2. `scripts/sync-npm-version.sh` - Added NAPI version sync
3. `Cargo.toml` - Version 0.1.9
4. `npm/package.json` - Version 0.1.9
5. `npm/dotenvage-napi/Cargo.toml` - Dependency version 0.1.9

### Business-Composer

- Currently using local link (temporary)
- Will be updated to `^0.1.9` once published

## Verification

Once v0.1.9 is published to npm, verify with:

```bash
npm view @dotenvage/node@0.1.9 dist.tarball
```

The package should include all 7 platform binaries.

## References

- [npm Trusted Publishers](https://docs.npmjs.com/trusted-publishers)
- [GitHub OIDC](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect)
- See `.github/OIDC_SETUP.md` for detailed setup instructions
