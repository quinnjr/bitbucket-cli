import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-repo-command',
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
          <span class="text-[var(--color-neutral-700)]">repo</span>
        </nav>
        <div class="flex items-center gap-4 mb-4">
          <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-xl flex items-center justify-center text-2xl">
            📁
          </div>
          <div>
            <h1 class="text-3xl font-bold text-[var(--color-neutral-900)]">bitbucket repo</h1>
            <p class="text-[var(--color-neutral-400)]">Manage repositories</p>
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
export class RepoCommandComponent {
  subcommands = [
    { name: 'list', description: 'List repositories in a workspace' },
    { name: 'view', description: 'View repository details' },
    { name: 'clone', description: 'Clone a repository' },
    { name: 'create', description: 'Create a new repository' },
    { name: 'update', description: 'Update repository settings' },
    { name: 'move', description: 'Move a repository to a different project in its workspace' },
    { name: 'fork', description: 'Fork a repository' },
    { name: 'delete', description: 'Delete a repository' },
    { name: 'download', description: "Manage repository downloads (upload, list, delete artifacts)" },
  ];

  examples = [
    { title: 'List repositories', command: 'bitbucket repo list myworkspace' },
    { title: 'View repository', command: 'bitbucket repo view myworkspace/myrepo' },
    { title: 'Clone repository', command: 'bitbucket repo clone myworkspace/myrepo' },
    { title: 'Create repository', command: 'bitbucket repo create myworkspace myrepo --description "My new repo"' },
    { title: 'Update repository', command: 'bitbucket repo update myworkspace/myrepo --description "New description" --private' },
    { title: 'Move repository to a project', command: 'bitbucket repo move myworkspace/myrepo PROJ' },
  ];
}
