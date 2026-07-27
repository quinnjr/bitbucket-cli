# Authentication Migration Guide

## v0.3.0 - OAuth 2.0 First with API Key Fallback

Bitbucket CLI now prioritizes OAuth 2.0 authentication with API keys as a fallback, reflecting Atlassian's deprecation of app passwords.

### What Changed

#### Authentication Methods
- **OAuth 2.0** is now the PREFERRED method (more secure, automatic token refresh)
- **API Keys** (HTTP access tokens) replace deprecated app passwords as fallback
- **App Passwords** are NO LONGER SUPPORTED (deprecated by Atlassian)

#### Breaking Changes

1. **Renamed Types**
   - `Credential::AppPassword` → `Credential::ApiKey`
   - The enum variant order changed - OAuth is now listed first

2. **CLI Arguments**
   ```bash
   # OLD (v0.2.x)
   bitbucket auth login                    # Used app passwords
   bitbucket auth login --app-password     # Explicit app password
   
   # NEW (v0.3.0+)
   bitbucket auth login                    # Interactive prompt (prefers OAuth)
   bitbucket auth login --oauth            # Explicit OAuth
   bitbucket auth login --api-key          # Explicit API key
   ```

3. **Environment Variables**
   - New: `BITBUCKET_CLIENT_ID` - OAuth client ID
   - New: `BITBUCKET_CLIENT_SECRET` - OAuth client secret

### Migration Steps

#### For Individual Users

**If you previously used app passwords:**

1. Remove old credentials:
   ```bash
   bitbucket auth logout
   ```

2. Set up OAuth (recommended):
   ```bash
   # Create OAuth consumer at:
   # https://bitbucket.org/[workspace]/workspace/settings/oauth-consumers/new
   # 
   # Then authenticate:
   bitbucket auth login --oauth
   ```

3. **OR** use HTTP access tokens (fallback):
   ```bash
   # Create token at:
   # https://bitbucket.org/account/settings/app-passwords/
   #
   # Then authenticate:
   bitbucket auth login --api-key
   ```

#### For CI/CD Pipelines

**Recommended: Use API Keys**

API keys are better suited for automated environments:

```bash
# In your CI/CD pipeline
bitbucket auth login --api-key <<EOF
$BITBUCKET_USERNAME
$BITBUCKET_API_KEY
EOF
```

**Alternative: OAuth with Client Credentials**

If you have an OAuth app set up:

```bash
export BITBUCKET_CLIENT_ID="your_client_id"
export BITBUCKET_CLIENT_SECRET="your_client_secret"
bitbucket auth login --oauth
```

### Code Changes

If you're using bitbucket-cli as a library:

```rust
// OLD (v0.2.x)
use bitbucket_cli::auth::{Credential, AppPasswordAuth};

let credential = Credential::AppPassword {
    username: "user".to_string(),
    app_password: "password".to_string(),
};

// NEW (v0.3.0+)
use bitbucket_cli::auth::{Credential, ApiKeyAuth, OAuthFlow};

// Option 1: API Key
let credential = Credential::ApiKey {
    username: "user".to_string(),
    api_key: "token".to_string(),
};

// Option 2: OAuth (preferred)
let credential = Credential::OAuth {
    access_token: "access_token".to_string(),
    refresh_token: Some("refresh_token".to_string()),
    expires_at: Some(timestamp),
};
```

### Benefits of OAuth 2.0

- **Automatic token refresh** - No manual re-authentication needed
- **Better security** - Access tokens expire and can be revoked
- **No password exposure** - Never store or transmit your password
- **Granular permissions** - Fine-grained control over access
- **Modern standard** - Industry best practice

### Benefits of API Keys

- **Simple for automation** - No browser interaction required
- **Long-lived** - Don't expire unless revoked
- **CI/CD friendly** - Easy to configure in pipelines
- **Direct authentication** - No OAuth flow complexity

### Troubleshooting

#### "Invalid username or app password"

You're trying to use an old app password. Update to API keys or OAuth:

```bash
bitbucket auth logout
bitbucket auth login --api-key  # or --oauth
```

#### "OAuth Client ID is required"

When using OAuth, you need to create an OAuth consumer first:

1. Go to https://bitbucket.org/[workspace]/workspace/settings/oauth-consumers/new
2. Set callback URL to **ONE** of these (the CLI tries them in order):
   - `http://127.0.0.1:8080/callback`
   - `http://127.0.0.1:3000/callback`
   - `http://127.0.0.1:8888/callback`
   - `http://127.0.0.1:9000/callback`
3. Select permissions:
   - Account (Read)
   - Repositories (Read)
   - Pull requests (Read, Write)
   - Issues (Read, Write)
   - Pipelines (Read, Write)
4. Save and copy Key (Client ID) and Secret

#### "Credentials may be invalid"

Run `bitbucket auth status` to check your authentication method and token status.

For OAuth tokens that need refresh:
```bash
# The CLI will automatically refresh on next API call
# Or force refresh by logging in again
bitbucket auth login --oauth
```

### FAQ

**Q: Can I still use my old app password?**

A: No. Atlassian has deprecated app passwords. You must migrate to API keys or OAuth.

**Q: What's the difference between API keys and HTTP access tokens?**

A: They're the same thing. Bitbucket calls them "HTTP access tokens" in the UI.

**Q: Which method should I use?**

A: 
- **Individual developers**: OAuth 2.0 (best experience)
- **CI/CD pipelines**: API keys (simplest setup)
- **Long-running services**: OAuth 2.0 with refresh tokens

**Q: Will my existing credentials still work?**

A: No. You must re-authenticate using the new methods after upgrading to v0.3.0.

**Q: Can I use both OAuth and API keys?**

A: Only one credential is stored at a time. You can switch between them using `bitbucket auth logout` and `bitbucket auth login`.

### Support

For issues or questions:
- GitHub Issues: https://github.com/quinnjr/bitbucket-cli/issues
- Documentation: https://quinnjr.github.io/bitbucket-cli/

### Timeline

- **v0.2.x**: App passwords supported (deprecated by Atlassian)
- **v0.3.0**: OAuth 2.0 preferred, API keys as fallback
- **Future**: App password support completely removed
