import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { IconDefinition } from '@fortawesome/fontawesome-svg-core';
import {
  faFolder, faCodeBranch, faBug, faBolt, faBuilding, faTerminal, faShieldAlt,
  faArrowRight, faCopy, faChevronRight, faKey
} from '@fortawesome/free-solid-svg-icons';
import { faGithub, faBitbucket } from '@fortawesome/free-brands-svg-icons';

interface Feature {
  icon: IconDefinition;
  title: string;
  description: string;
}

interface Command {
  name: string;
  path: string;
  icon: IconDefinition;
  description: string;
}

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [CommonModule, RouterLink, FontAwesomeModule],
  template: `
    <div class="max-w-4xl mx-auto px-6 py-12">
      <!-- Hero Section -->
      <div class="text-center mb-16">
        <div class="inline-flex items-center justify-center w-20 h-20 bg-[var(--color-bitbucket-blue)] rounded-2xl mb-6 shadow-lg">
          <fa-icon [icon]="faBitbucket" class="text-white text-4xl"></fa-icon>
        </div>
        <h1 class="text-4xl font-bold text-[var(--color-neutral-900)] mb-4">
          Bitbucket CLI
        </h1>
        <p class="text-xl text-[var(--color-neutral-400)] max-w-2xl mx-auto mb-8">
          A powerful command-line interface for Bitbucket Cloud. Manage repositories, pull requests, issues, and pipelines directly from your terminal.
        </p>
        <div class="flex flex-wrap justify-center gap-4">
          <a
            routerLink="/installation"
            class="inline-flex items-center gap-2 px-6 py-3 bg-[var(--color-bitbucket-blue)] text-white font-medium rounded-lg hover:bg-[var(--color-bitbucket-blue-dark)] transition-colors shadow-md"
          >
            Get Started
            <fa-icon [icon]="faArrowRight"></fa-icon>
          </a>
          <a
            href="https://github.com/pegasusheavy/bitbucket-cli"
            target="_blank"
            class="inline-flex items-center gap-2 px-6 py-3 bg-white text-[var(--color-neutral-700)] font-medium rounded-lg border border-[var(--color-neutral-40)] hover:bg-[var(--color-neutral-20)] transition-colors"
          >
            <fa-icon [icon]="faGithub" class="text-lg"></fa-icon>
            View on GitHub
          </a>
        </div>
      </div>

      <!-- Quick Install -->
      <div class="bg-[var(--color-neutral-900)] rounded-xl p-6 mb-16">
        <div class="flex items-center justify-between mb-4">
          <span class="text-[var(--color-neutral-100)] text-sm font-medium">Quick Install</span>
          <button
            class="text-[var(--color-neutral-100)] hover:text-white text-sm flex items-center gap-1 transition-colors"
            (click)="copyToClipboard('cargo install bitbucket-cli')"
          >
            <fa-icon [icon]="faCopy"></fa-icon>
            Copy
          </button>
        </div>
        <code class="text-[var(--color-bitbucket-blue-light)] font-mono text-lg">
          cargo install bitbucket-cli
        </code>
      </div>

      <!-- Features Grid -->
      <div class="mb-16">
        <h2 class="text-2xl font-bold text-[var(--color-neutral-900)] mb-8 text-center">Features</h2>
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          @for (feature of features; track feature.title) {
            <div class="bg-white rounded-xl p-6 border border-[var(--color-neutral-30)] hover:shadow-lg transition-shadow">
              <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center mb-4">
                <fa-icon [icon]="feature.icon" class="text-[var(--color-bitbucket-blue)] text-xl"></fa-icon>
              </div>
              <h3 class="text-lg font-semibold text-[var(--color-neutral-800)] mb-2">{{ feature.title }}</h3>
              <p class="text-[var(--color-neutral-400)] text-sm">{{ feature.description }}</p>
            </div>
          }
        </div>
      </div>

      <!-- Command Overview -->
      <div class="mb-16">
        <h2 class="text-2xl font-bold text-[var(--color-neutral-900)] mb-8 text-center">Command Overview</h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] overflow-hidden">
          @for (cmd of commands; track cmd.name) {
            <a
              [routerLink]="cmd.path"
              class="flex items-center gap-4 p-4 hover:bg-[var(--color-neutral-10)] transition-colors border-b border-[var(--color-neutral-30)] last:border-b-0"
            >
              <div class="w-10 h-10 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center flex-shrink-0">
                <fa-icon [icon]="cmd.icon" class="text-[var(--color-bitbucket-blue)]"></fa-icon>
              </div>
              <div class="flex-1 min-w-0">
                <div class="font-mono text-[var(--color-bitbucket-blue)] font-medium">bitbucket {{ cmd.name }}</div>
                <div class="text-sm text-[var(--color-neutral-400)] truncate">{{ cmd.description }}</div>
              </div>
              <fa-icon [icon]="faChevronRight" class="text-[var(--color-neutral-100)] flex-shrink-0"></fa-icon>
            </a>
          }
        </div>
      </div>

      <!-- Call to Action -->
      <div class="bg-gradient-to-r from-[var(--color-bitbucket-blue)] to-[var(--color-bitbucket-blue-dark)] rounded-2xl p-8 text-center text-white">
        <h2 class="text-2xl font-bold mb-4">Ready to get started?</h2>
        <p class="text-[var(--color-bitbucket-blue-100)] mb-6 max-w-lg mx-auto">
          Install the Bitbucket CLI and streamline your development workflow today.
        </p>
        <a
          routerLink="/installation"
          class="inline-flex items-center gap-2 px-6 py-3 bg-white text-[var(--color-bitbucket-blue)] font-medium rounded-lg hover:bg-[var(--color-neutral-10)] transition-colors"
        >
          View Installation Guide
          <fa-icon [icon]="faArrowRight"></fa-icon>
        </a>
      </div>
    </div>
  `
})
export class HomeComponent {
  // Icons
  faBitbucket = faBitbucket;
  faGithub = faGithub;
  faArrowRight = faArrowRight;
  faCopy = faCopy;
  faChevronRight = faChevronRight;

  features: Feature[] = [
    {
      icon: faFolder,
      title: 'Repository Management',
      description: 'List, view, clone, create, and manage your Bitbucket repositories.'
    },
    {
      icon: faCodeBranch,
      title: 'Pull Requests',
      description: 'Create, review, merge, approve, and manage pull requests efficiently.'
    },
    {
      icon: faBug,
      title: 'Issue Tracking',
      description: 'Create, view, comment on, and manage issues directly from the CLI.'
    },
    {
      icon: faBolt,
      title: 'Pipelines',
      description: 'Trigger, monitor, and manage your CI/CD pipelines with ease.'
    },
    {
      icon: faTerminal,
      title: 'Interactive TUI',
      description: 'Beautiful terminal UI for browsing repositories, PRs, and more.'
    },
    {
      icon: faShieldAlt,
      title: 'Secure Auth',
      description: 'Support for OAuth 2.0 and API keys with secure keyring storage.'
    }
  ];

  commands: Command[] = [
    { name: 'auth', path: '/commands/auth', icon: faKey, description: 'Manage authentication with Bitbucket' },
    { name: 'repo', path: '/commands/repo', icon: faFolder, description: 'Manage repositories' },
    { name: 'pr', path: '/commands/pr', icon: faCodeBranch, description: 'Manage pull requests' },
    { name: 'issue', path: '/commands/issue', icon: faBug, description: 'Manage issues' },
    { name: 'pipeline', path: '/commands/pipeline', icon: faBolt, description: 'Manage pipelines' },
    { name: 'workspace', path: '/commands/workspace', icon: faBuilding, description: 'Manage workspaces' },
    { name: 'tui', path: '/tui', icon: faTerminal, description: 'Launch the interactive terminal UI' },
  ];

  copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }
}
