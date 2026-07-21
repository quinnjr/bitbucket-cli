import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { IconDefinition } from '@fortawesome/fontawesome-svg-core';
import {
  faHome, faDownload, faLock, faCog, faKey, faFolder, faCodeBranch,
  faBug, faBolt, faBuilding, faTerminal
} from '@fortawesome/free-solid-svg-icons';
import { faGithub, faBitbucket } from '@fortawesome/free-brands-svg-icons';

interface NavItem {
  label: string;
  path: string;
  icon: IconDefinition;
}

interface NavSection {
  label: string;
  items: NavItem[];
}

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive, FontAwesomeModule],
  template: `
    <aside class="w-64 bg-[var(--color-sidebar-bg)] flex flex-col h-full">
      <!-- Logo -->
      <div class="p-4 border-b border-[var(--color-bitbucket-blue)]">
        <a routerLink="/" class="flex items-center gap-3">
          <div class="w-8 h-8 bg-white rounded flex items-center justify-center">
            <fa-icon [icon]="faBitbucket" class="text-[var(--color-bitbucket-blue)] text-lg"></fa-icon>
          </div>
          <span class="text-white font-semibold text-lg">Bitbucket CLI</span>
        </a>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 overflow-y-auto py-4">
        @for (section of navSections; track section.label) {
          <div class="mb-4">
            <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-sidebar-text-muted)]">
              {{ section.label }}
            </div>
            @for (item of section.items; track item.path) {
              <a
                [routerLink]="item.path"
                routerLinkActive="bg-[var(--color-sidebar-hover)] border-l-2 border-white"
                [routerLinkActiveOptions]="{ exact: item.path === '/' }"
                class="flex items-center gap-3 px-4 py-2 text-[var(--color-sidebar-text)] hover:bg-[var(--color-sidebar-hover)] transition-colors"
              >
                <fa-icon [icon]="item.icon" class="w-4 text-center"></fa-icon>
                <span class="text-sm">{{ item.label }}</span>
              </a>
            }
          </div>
        }
      </nav>

      <!-- Footer -->
      <div class="p-4 border-t border-[var(--color-bitbucket-blue)]">
        <a
          href="https://github.com/pegasusheavy/bitbucket-cli"
          target="_blank"
          class="flex items-center gap-2 text-[var(--color-sidebar-text-muted)] hover:text-white text-sm transition-colors"
        >
          <fa-icon [icon]="faGithub" class="text-lg"></fa-icon>
          <span>View on GitHub</span>
        </a>
      </div>
    </aside>
  `
})
export class SidebarComponent {
  // Brand icons
  faBitbucket = faBitbucket;
  faGithub = faGithub;

  navSections: NavSection[] = [
    {
      label: 'Getting Started',
      items: [
        { label: 'Introduction', path: '/', icon: faHome },
        { label: 'Installation', path: '/installation', icon: faDownload },
        { label: 'Authentication', path: '/authentication', icon: faLock },
        { label: 'Configuration', path: '/configuration', icon: faCog },
      ]
    },
    {
      label: 'Commands',
      items: [
        { label: 'auth', path: '/commands/auth', icon: faKey },
        { label: 'repo', path: '/commands/repo', icon: faFolder },
        { label: 'pr', path: '/commands/pr', icon: faCodeBranch },
        { label: 'issue', path: '/commands/issue', icon: faBug },
        { label: 'pipeline', path: '/commands/pipeline', icon: faBolt },
        { label: 'workspace', path: '/commands/workspace', icon: faBuilding },
      ]
    },
    {
      label: 'Advanced',
      items: [
        { label: 'TUI Mode', path: '/tui', icon: faTerminal },
      ]
    }
  ];
}
