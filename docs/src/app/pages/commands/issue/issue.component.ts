import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-issue-command',
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
          <span>Commands</span>
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
          <span class="text-[var(--color-neutral-700)]">issue</span>
        </nav>
        <div class="flex items-center gap-4 mb-4">
          <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-xl flex items-center justify-center text-2xl">
            🐛
          </div>
          <div>
            <h1 class="text-3xl font-bold text-[var(--color-neutral-900)]">bitbucket issue</h1>
            <p class="text-[var(--color-neutral-400)]">Manage issues</p>
          </div>
        </div>
      </div>

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
export class IssueCommandComponent {
  subcommands = [
    { name: 'list', description: 'List issues (--state, --kind, --priority, --assignee, --reporter, -q/--query, --sort, --page)' },
    { name: 'view', description: 'View issue details (--comments to include the discussion)' },
    { name: 'create', description: 'Create a new issue (--title, --body, --kind, --priority, --assignee, --component, --milestone, --version)' },
    { name: 'edit', description: "Edit an issue's fields (--title, --body, --kind, --priority, --assignee, --state)" },
    { name: 'delete', description: 'Delete an issue' },
    { name: 'close', description: 'Close an issue' },
    { name: 'reopen', description: 'Reopen an issue' },
    { name: 'comment', description: 'Add a comment to an issue' },
    { name: 'comments', description: 'List comments on an issue' },
    { name: 'edit-comment', description: 'Edit a comment on an issue' },
    { name: 'delete-comment', description: 'Delete a comment from an issue' },
    { name: 'changes', description: 'List the change log of an issue' },
    { name: 'vote', description: 'Vote for an issue' },
    { name: 'unvote', description: 'Remove your vote from an issue' },
    { name: 'watch', description: 'Watch an issue' },
    { name: 'unwatch', description: 'Stop watching an issue' },
    { name: 'components', description: 'List the components defined in the issue tracker' },
    { name: 'milestones', description: 'List the milestones defined in the issue tracker' },
    { name: 'versions', description: 'List the versions defined in the issue tracker' },
    { name: 'attachment', description: 'Manage issue attachments (list, add, delete)' },
  ];

  examples = [
    { title: 'List issues', command: 'bitbucket issue list myworkspace/myrepo' },
    { title: 'List critical open bugs', command: 'bitbucket issue list myworkspace/myrepo --state open --kind bug --priority critical' },
    { title: 'Create an issue', command: 'bitbucket issue create myworkspace/myrepo --title "Bug report" --kind bug --priority major --component backend' },
    { title: 'View an issue with its comments', command: 'bitbucket issue view myworkspace/myrepo 42 --comments' },
    { title: 'Edit an issue', command: 'bitbucket issue edit myworkspace/myrepo 42 --priority blocker --assignee 557058:aaaa-bbbb' },
    { title: 'Attach a file', command: 'bitbucket issue attachment add myworkspace/myrepo 42 screenshot.png' },
    { title: 'Close an issue', command: 'bitbucket issue close myworkspace/myrepo 42' },
  ];
}
