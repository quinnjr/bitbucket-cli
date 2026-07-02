import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pr-command',
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
          <span class="text-[var(--color-neutral-700)]">pr</span>
        </nav>
        <div class="flex items-center gap-4 mb-4">
          <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-xl flex items-center justify-center text-2xl">
            🔀
          </div>
          <div>
            <h1 class="text-3xl font-bold text-[var(--color-neutral-900)]">bitbucket pr</h1>
            <p class="text-[var(--color-neutral-400)]">Manage pull requests</p>
          </div>
        </div>
      </div>

      <!-- Subcommands -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Subcommands</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] overflow-hidden">
          @for (cmd of subcommands; track cmd.name) {
            <div class="p-4 border-b border-[var(--color-neutral-30)] last:border-b-0">
              <div class="flex items-start justify-between">
                <div>
                  <code class="text-[var(--color-bitbucket-blue)] font-mono font-medium">{{ cmd.name }}</code>
                  <p class="text-sm text-[var(--color-neutral-400)] mt-1">{{ cmd.description }}</p>
                </div>
              </div>
            </div>
          }
        </div>
      </section>

      <!-- Examples -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Examples</h2>
        <div class="space-y-4">
          @for (example of examples; track example.title) {
            <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
              <h3 class="font-medium text-[var(--color-neutral-800)] mb-2">{{ example.title }}</h3>
              <div class="bg-[var(--color-neutral-900)] rounded-lg p-4">
                <code class="text-[var(--color-bitbucket-blue-light)] font-mono text-sm">{{ example.command }}</code>
              </div>
              @if (example.description) {
                <p class="text-sm text-[var(--color-neutral-400)] mt-3">{{ example.description }}</p>
              }
            </div>
          }
        </div>
      </section>

      <!-- Options -->
      <section>
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Common Options</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] overflow-hidden">
          <table class="w-full">
            <thead class="bg-[var(--color-neutral-10)]">
              <tr>
                <th class="text-left px-4 py-3 text-sm font-medium text-[var(--color-neutral-600)]">Option</th>
                <th class="text-left px-4 py-3 text-sm font-medium text-[var(--color-neutral-600)]">Description</th>
              </tr>
            </thead>
            <tbody>
              @for (option of options; track option.flag) {
                <tr class="border-t border-[var(--color-neutral-30)]">
                  <td class="px-4 py-3">
                    <code class="text-[var(--color-bitbucket-blue)] font-mono text-sm">{{ option.flag }}</code>
                  </td>
                  <td class="px-4 py-3 text-sm text-[var(--color-neutral-600)]">{{ option.description }}</td>
                </tr>
              }
            </tbody>
          </table>
        </div>
      </section>
    </div>
  `
})
export class PrCommandComponent {
  subcommands = [
    { name: 'list', description: 'List pull requests for a repository' },
    { name: 'view', description: 'View pull request details' },
    { name: 'create', description: 'Create a new pull request' },
    { name: 'edit', description: 'Edit a pull request title and/or description' },
    { name: 'merge', description: 'Merge a pull request' },
    { name: 'approve', description: 'Approve a pull request' },
    { name: 'decline', description: 'Decline a pull request' },
    { name: 'checkout', description: 'Checkout a PR branch locally' },
    { name: 'diff', description: 'View the diff for a pull request' },
    { name: 'comment', description: 'Add a comment to a pull request' },
  ];

  examples = [
    {
      title: 'List open pull requests',
      command: 'bitbucket pr list myworkspace/myrepo',
      description: 'Lists all open pull requests in the specified repository.'
    },
    {
      title: 'Create a pull request',
      command: 'bitbucket pr create myworkspace/myrepo --title "Add feature" --source feature-branch --destination main',
      description: 'Creates a new pull request from feature-branch to main.'
    },
    {
      title: 'View pull request details',
      command: 'bitbucket pr view myworkspace/myrepo 42',
      description: 'Shows detailed information about PR #42.'
    },
    {
      title: 'Edit a pull request description',
      command: 'bitbucket pr edit myworkspace/myrepo 42',
      description: 'Opens $EDITOR prefilled with the current description. Pass --body to set it directly, or --title to change the title.'
    },
    {
      title: 'Merge a pull request',
      command: 'bitbucket pr merge myworkspace/myrepo 42 --strategy squash',
      description: 'Merges PR #42 using squash merge strategy.'
    },
    {
      title: 'Checkout PR locally',
      command: 'bitbucket pr checkout myworkspace/myrepo 42',
      description: 'Fetches and checks out the branch for PR #42.'
    },
  ];

  options = [
    { flag: '--state <STATE>', description: 'Filter by state (open, merged, declined)' },
    { flag: '--limit <N>', description: 'Number of results to show (default: 25)' },
    { flag: '--web', description: 'Open in browser instead of showing in terminal' },
    { flag: '--strategy <STRATEGY>', description: 'Merge strategy (merge-commit, squash, fast-forward)' },
    { flag: '--title <TITLE>', description: 'New title when editing a pull request' },
    { flag: '--body <BODY>', description: 'New description when editing (skips the editor)' },
  ];
}
