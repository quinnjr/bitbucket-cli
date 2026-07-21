import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-configuration',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="max-w-4xl mx-auto px-6 py-12">
      <div class="mb-12">
        <nav class="flex items-center gap-2 text-sm text-[var(--color-neutral-400)] mb-4">
          <a routerLink="/" class="hover:text-[var(--color-bitbucket-blue)]">Docs</a>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
          <span class="text-[var(--color-neutral-700)]">Configuration</span>
        </nav>
        <h1 class="text-3xl font-bold text-[var(--color-neutral-900)] mb-4">Configuration</h1>
        <p class="text-lg text-[var(--color-neutral-400)]">
          Customize the Bitbucket CLI to match your workflow.
        </p>
      </div>

      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Configuration File</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
          <p class="text-[var(--color-neutral-600)] mb-4">
            Configuration is stored in <code class="bg-[var(--color-neutral-20)] px-2 py-0.5 rounded">~/.config/bitbucket-cli/config.toml</code>
          </p>
          <div class="bg-[var(--color-neutral-900)] rounded-lg p-4">
            <pre class="text-sm text-[var(--color-neutral-100)] font-mono overflow-x-auto"><code>[auth]
username = "your-username"
default_workspace = "your-workspace"

[display]
color = true
pager = true</code></pre>
          </div>
        </div>
      </section>

      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Configuration Options</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] overflow-hidden">
          @for (option of options; track option.key) {
            <div class="p-4 border-b border-[var(--color-neutral-30)] last:border-b-0">
              <code class="text-[var(--color-bitbucket-blue)] font-mono font-medium">{{ option.key }}</code>
              <p class="text-sm text-[var(--color-neutral-400)] mt-1">{{ option.description }}</p>
              <p class="text-xs text-[var(--color-neutral-300)] mt-1">Default: <code>{{ option.default }}</code></p>
            </div>
          }
        </div>
      </section>

      <section>
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Global Options</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
          <p class="text-[var(--color-neutral-600)] mb-4">
            These options can be passed to any command:
          </p>
          <div class="space-y-3">
            <div>
              <code class="text-[var(--color-bitbucket-blue)] font-mono">-w, --workspace &lt;WORKSPACE&gt;</code>
              <p class="text-sm text-[var(--color-neutral-400)] mt-1">Override the default workspace</p>
            </div>
            <div>
              <code class="text-[var(--color-bitbucket-blue)] font-mono">-r, --repo &lt;REPO&gt;</code>
              <p class="text-sm text-[var(--color-neutral-400)] mt-1">Override the repository</p>
            </div>
          </div>
        </div>
      </section>
    </div>
  `
})
export class ConfigurationComponent {
  options = [
    { key: 'auth.username', description: 'Your Bitbucket username (set automatically on login)', default: 'none' },
    { key: 'auth.default_workspace', description: 'Default workspace for commands', default: 'none' },
    { key: 'defaults.workspace', description: 'Legacy fallback for the default workspace; auth.default_workspace takes precedence', default: 'none' },
    { key: 'display.color', description: 'Enable colored output', default: 'true' },
    { key: 'display.pager', description: 'Use pager for long output', default: 'true' },
  ];
}
