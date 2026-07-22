import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pipeline-command',
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
          <span class="text-[var(--color-neutral-700)]">pipeline</span>
        </nav>
        <div class="flex items-center gap-4 mb-4">
          <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-xl flex items-center justify-center text-2xl">
            ⚡
          </div>
          <div>
            <h1 class="text-3xl font-bold text-[var(--color-neutral-900)]">bitbucket pipeline</h1>
            <p class="text-[var(--color-neutral-400)]">Manage pipelines</p>
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
export class PipelineCommandComponent {
  subcommands = [
    { name: 'list', description: 'List pipelines (--status, --target-branch, --sort, --page)' },
    { name: 'view', description: 'View pipeline details (--logs, --step, --full-logs)' },
    { name: 'trigger', description: 'Trigger a new pipeline (--branch, --commit, --pipeline, --var, --secured-var, --wait)' },
    { name: 'stop', description: 'Stop a running pipeline (--build or --uuid)' },
    { name: 'rerun', description: 'Re-run a pipeline by triggering an equivalent one' },
    { name: 'config', description: 'View or update the repository pipelines configuration (--enable, --disable, --next-build-number)' },
    { name: 'variable', description: 'Manage repository pipeline variables (list, set, delete)' },
    { name: 'workspace-variable', description: 'Manage workspace-level pipeline variables (list, set, delete)' },
    { name: 'schedule', description: 'Manage pipeline schedules (list, create, delete)' },
    { name: 'cache', description: 'Manage pipeline dependency caches (list, delete)' },
  ];

  examples = [
    { title: 'List recent pipelines', command: 'bitbucket pipeline list myworkspace/myrepo' },
    { title: 'List completed pipelines on main', command: 'bitbucket pipeline list myworkspace/myrepo --status COMPLETED --target-branch main' },
    { title: 'Trigger pipeline on main', command: 'bitbucket pipeline trigger myworkspace/myrepo --branch main' },
    { title: 'Trigger with variables and wait', command: 'bitbucket pipeline trigger myworkspace/myrepo --branch main --var ENV=staging --secured-var TOKEN=s3cret --wait' },
    { title: 'Trigger a custom pipeline on a commit', command: 'bitbucket pipeline trigger myworkspace/myrepo --commit abc123 --pipeline deploy' },
    { title: 'View pipeline details', command: 'bitbucket pipeline view myworkspace/myrepo --build 123' },
    { title: 'View full logs for one step', command: 'bitbucket pipeline view myworkspace/myrepo --build 123 --step 2 --full-logs' },
    { title: 'Re-run a pipeline', command: 'bitbucket pipeline rerun myworkspace/myrepo --build 123' },
    { title: 'Set a secured pipeline variable', command: 'bitbucket pipeline variable set myworkspace/myrepo DEPLOY_KEY abc123 --secured' },
    { title: 'Create a nightly schedule', command: 'bitbucket pipeline schedule create myworkspace/myrepo --branch main --cron "0 0 12 * * ? *"' },
  ];
}
