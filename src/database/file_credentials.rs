// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

pub fn get() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        // ── Cloud Providers ──────────────────────────────────────────────
        (".aws/credentials", vec!["aws", "terraform", "pulumi", "ansible"]),
        (".aws/config", vec!["aws", "terraform", "pulumi", "ansible"]),
        (
            ".config/gcloud/credentials.db",
            vec!["gcloud", "gsutil", "terraform", "pulumi"],
        ),
        (".config/gcloud/access_tokens.db", vec!["gcloud", "gsutil"]),
        (
            ".config/gcloud/application_default_credentials.json",
            vec!["gcloud", "gsutil", "terraform"],
        ),
        (".config/gcloud/legacy_credentials/*/adc.json", vec!["gcloud"]),
        (".azure/accessTokens.json", vec!["az", "terraform", "pulumi"]),
        (".azure/clouds.config", vec!["az"]),
        (".config/doctl/config.yaml", vec!["doctl", "terraform"]),
        (".config/linode-cli", vec!["linode-cli"]),
        (".config/hcloud/cli.toml", vec!["hcloud"]),
        (".config/vultr-cli/config.yaml", vec!["vultr-cli"]),
        (".config/scaleway/config.yaml", vec!["scw"]),
        (".civo.json", vec!["civo"]),
        (".oci/config", vec!["oci", "terraform"]),
        (".aliyun/config.json", vec!["aliyun"]),
        (".config/ibmcloud/config.json", vec!["ibmcloud"]),
        (".config/ibmcloud/.bluemix/config.json", vec!["ibmcloud"]),
        // ── Container / Kubernetes ───────────────────────────────────────
        (".kube/config", vec!["kubectl", "helm", "k9s", "terraform", "flux"]),
        (".config/helm/repositories.yaml", vec!["helm"]),
        (".docker/config.json", vec!["docker", "podman", "nerdctl"]),
        (".config/containers/auth.json", vec!["podman", "buildah", "skopeo"]),
        // ── Source Control / Forges ──────────────────────────────────────
        (".config/gh/hosts.yml", vec!["gh"]),
        (".config/gh/config.yml", vec!["gh"]),
        (".gitconfig", vec!["git"]),
        (".git-credentials", vec!["git"]),
        (".config/hub", vec!["hub"]),
        (".config/glab-cli/config.yml", vec!["glab"]),
        // ── Infrastructure as Code ───────────────────────────────────────
        (".config/terraform.d/credentials.tfrc.json", vec!["terraform"]),
        (".terraform.d/credentials.tfrc.json", vec!["terraform"]),
        (".config/pulumi/credentials.json", vec!["pulumi"]),
        (".vagrant.d/data/machine-index/index", vec!["vagrant"]),
        // ── AI / LLM Providers ───────────────────────────────────────────
        (".config/anthropic/config.json", vec!["claude"]),
        (".claude.json", vec!["claude"]),
        (".claude.json.backup", vec!["claude"]),
        (".config/openai/config.json", vec!["openai"]),
        (".config/cohere/config.json", vec!["cohere"]),
        (".local/share/opencode/auth.json", vec!["opencode"]),
        // ── Package Registries ───────────────────────────────────────────
        (".npmrc", vec!["npm", "pnpm", "yarn"]),
        (".config/npm/npmrc", vec!["npm"]),
        (".yarnrc.yml", vec!["yarn"]),
        (".config/pypoetry/auth.toml", vec!["poetry"]),
        (".config/uv/uv.toml", vec!["uv"]),
        (".cargo/credentials.toml", vec!["cargo"]),
        (".config/composer/auth.json", vec!["composer"]),
        (".gem/credentials", vec!["gem", "bundle"]),
        (".config/pypi/pypirc", vec!["twine", "poetry"]),
        (".pypirc", vec!["twine", "poetry"]),
        (".netrc", vec!["curl", "wget", "ftp"]),
        // ── Databases ────────────────────────────────────────────────────
        (".config/pgpass", vec!["psql", "pg_dump", "pg_restore"]),
        (".pgpass", vec!["psql", "pg_dump", "pg_restore"]),
        (".my.cnf", vec!["mysql", "mysqldump"]),
        (".config/mycli/config", vec!["mycli"]),
        (".config/litecli/config", vec!["litecli"]),
        (".rediscli_auth", vec!["redis-cli"]),
        (".config/mongosh/mongosh.conf", vec!["mongosh"]),
        // ── Password Managers / Secret Stores ────────────────────────────
        (".config/op/config", vec!["op"]), // 1Password CLI
        (".config/bitwarden-cli/data.json", vec!["bw"]),
        (".password-store/.gpg-id", vec!["pass", "gopass"]),
        (".config/gopass/config.yaml", vec!["gopass"]),
        // ── Communication / SaaS ─────────────────────────────────────────
        (".config/slack-term/config", vec!["slack-term"]),
        (".config/discord/*/Local Storage/*", vec!["discord"]),
        (".config/Slack/*/Cookies", vec!["slack"]),
        (".netrc", vec!["curl"]), // also catches Heroku, etc.
        (".config/heroku/netrc", vec!["heroku"]),
        // ── CI / CD ──────────────────────────────────────────────────────
        (".config/circleci/cli.yml", vec!["circleci"]),
        (".config/travis/config.yaml", vec!["travis"]),
        // ── Monitoring / Observability ───────────────────────────────────
        (".config/datadog/datadog.yaml", vec!["datadog-agent", "ddtrace-run"]),
        (".config/pagerduty-cli/config.json", vec!["pd"]),
        (".config/newrelic/credentials.json", vec!["newrelic"]),
        // ── Misc Developer Tools ─────────────────────────────────────────
        (".config/ngrok/ngrok.yml", vec!["ngrok"]),
        (".config/stripe/config.toml", vec!["stripe"]),
        (".config/vercel/auth.json", vec!["vercel"]),
        (".config/netlify/config.json", vec!["netlify"]),
        (".config/fly/config.yml", vec!["flyctl", "fly"]),
        (".config/railway/config.json", vec!["railway"]),
        (".config/supabase/access-token", vec!["supabase"]),
        (".config/planetscale/pscale.yaml", vec!["pscale"]),
        (".config/turso/settings.json", vec!["turso"]),
        (".config/wrangler/config", vec!["wrangler"]), // Cloudflare
        (".config/cloudflared/cert.pem", vec!["cloudflared"]),
        (".cloudflare.ini", vec!["certbot"]),
        (".config/sentry-cli/credentials", vec!["sentry-cli"]),
        (".config/ansible/vault_password_file", vec!["ansible", "ansible-vault"]),
    ]
}
