import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-user-command',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="max-w-4xl mx-auto px-6 py-12">
      <!-- Page Header -->
      <div class="mb-12">
        <nav class="flex items-center gap-2 text-sm text-[var(--color-neutral-400)] mb-4">
          <a routerLink="/" class="hover:text-[var(--color-bitbucket-blue)]">Docs</a>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
          <span>Commands</span>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
          <span class="text-[var(--color-neutral-700)]">user</span>
        </nav>
        <div class="flex items-center gap-4 mb-4">
          <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-xl flex items-center justify-center text-2xl">
            👤
          </div>
          <div>
            <h1 class="text-3xl font-bold text-[var(--color-neutral-900)]">bitbucket user</h1>
            <p class="text-[var(--color-neutral-400)]">Inspect Bitbucket user accounts</p>
          </div>
        </div>
      </div>

      <!-- Subcommands -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Subcommands</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] overflow-hidden">
          @for (cmd of subcommands; track cmd.name) {
            <div class="p-4 border-b border-[var(--color-neutral-30)] last:border-b-0">
              <code class="text-[var(--color-bitbucket-blue)] font-mono font-medium">{{ cmd.name }}</code>
              <p class="text-sm text-[var(--color-neutral-400)] mt-1">{{ cmd.description }}</p>
            </div>
          }
        </div>
      </section>

      <!-- Examples -->
      <section>
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Examples</h2>
        <div class="space-y-4">
          @for (example of examples; track example.title) {
            <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
              <h3 class="font-medium text-[var(--color-neutral-800)] mb-2">{{ example.title }}</h3>
              <div class="bg-[var(--color-neutral-900)] rounded-lg p-4">
                <code class="text-[var(--color-bitbucket-blue-light)] font-mono text-sm">{{ example.command }}</code>
              </div>
            </div>
          }
        </div>
      </section>
    </div>
  `
})
export class UserCommandComponent {
  subcommands = [
    { name: 'whoami', description: 'Show the currently authenticated user' },
    { name: 'view', description: "View a user's public profile by account ID or UUID (including braces)" },
    { name: 'emails', description: 'List email addresses on the authenticated account' },
  ];

  examples = [
    { title: 'Show the authenticated user', command: 'bitbucket user whoami' },
    { title: "View a user's profile", command: 'bitbucket user view 557058:aaaa-bbbb-cccc' },
    { title: 'List your email addresses', command: 'bitbucket user emails' },
    { title: 'Machine-readable identity for scripts', command: 'bitbucket user whoami --output json' },
  ];
}
