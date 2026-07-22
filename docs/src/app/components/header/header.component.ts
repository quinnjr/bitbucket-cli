import { Component, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { faBars, faSearch } from '@fortawesome/free-solid-svg-icons';
import { faGithub } from '@fortawesome/free-brands-svg-icons';

@Component({
  selector: 'app-header',
  standalone: true,
  imports: [CommonModule, FontAwesomeModule],
  template: `
    <header class="bg-white border-b border-[var(--color-neutral-30)] px-6 py-3">
      <div class="flex items-center justify-between">
        <!-- Mobile menu button -->
        <button
          class="lg:hidden p-2 rounded-md hover:bg-[var(--color-neutral-20)] transition-colors"
          (click)="toggleMobileMenu()"
        >
          <fa-icon [icon]="faBars" class="text-[var(--color-neutral-500)] text-xl"></fa-icon>
        </button>

        <!-- Search -->
        <div class="flex-1 max-w-xl mx-4">
          <div class="relative">
            <input
              type="text"
              placeholder="Search documentation..."
              class="w-full pl-10 pr-4 py-2 border border-[var(--color-neutral-40)] rounded-lg bg-[var(--color-neutral-10)] focus:outline-none focus:ring-2 focus:ring-[var(--color-bitbucket-blue)] focus:border-transparent text-sm"
            >
            <fa-icon [icon]="faSearch" class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-neutral-100)]"></fa-icon>
            <kbd class="absolute right-3 top-1/2 -translate-y-1/2 hidden sm:inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium text-[var(--color-neutral-200)] bg-[var(--color-neutral-20)] rounded border border-[var(--color-neutral-40)]">
              ⌘K
            </kbd>
          </div>
        </div>

        <!-- Right side actions -->
        <div class="flex items-center gap-4">
          <a
            href="https://github.com/pegasusheavy/bitbucket-cli"
            target="_blank"
            class="hidden sm:flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-[var(--color-neutral-500)] hover:text-[var(--color-neutral-800)] transition-colors"
          >
            <fa-icon [icon]="faGithub" class="text-lg"></fa-icon>
            GitHub
          </a>

          <span class="hidden sm:inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-[var(--color-bitbucket-blue-50)] text-[var(--color-bitbucket-blue)]">
            v1.0.0
          </span>
        </div>
      </div>
    </header>

    <!-- Mobile menu overlay -->
    @if (mobileMenuOpen()) {
      <div class="fixed inset-0 z-50 lg:hidden">
        <div class="fixed inset-0 bg-black/50" (click)="toggleMobileMenu()"></div>
        <nav class="fixed left-0 top-0 bottom-0 w-64 bg-[var(--color-sidebar-bg)] shadow-xl">
          <!-- Mobile sidebar content would go here -->
        </nav>
      </div>
    }
  `
})
export class HeaderComponent {
  faBars = faBars;
  faSearch = faSearch;
  faGithub = faGithub;

  mobileMenuOpen = signal(false);

  toggleMobileMenu() {
    this.mobileMenuOpen.update(v => !v);
  }
}
