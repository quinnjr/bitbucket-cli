import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-installation',
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
          <span class="text-[var(--color-neutral-700)]">Installation</span>
        </nav>
        <h1 class="text-3xl font-bold text-[var(--color-neutral-900)] mb-4">Installation</h1>
        <p class="text-lg text-[var(--color-neutral-400)]">
          Get started with Bitbucket CLI by installing it on your system.
        </p>
      </div>

      <!-- Prerequisites -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4 flex items-center gap-2">
          <span class="w-8 h-8 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center text-sm">1</span>
          Prerequisites
        </h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
          <p class="text-[var(--color-neutral-600)] mb-4">
            Before installing, ensure you have the following:
          </p>
          <ul class="space-y-3">
            <li class="flex items-start gap-3">
              <svg class="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
              </svg>
              <div>
                <span class="font-medium text-[var(--color-neutral-800)]">Rust toolchain</span>
                <span class="text-[var(--color-neutral-400)]"> - Install from </span>
                <a href="https://rustup.rs" target="_blank" class="text-[var(--color-bitbucket-blue)] hover:underline">rustup.rs</a>
              </div>
            </li>
            <li class="flex items-start gap-3">
              <svg class="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
              </svg>
              <div>
                <span class="font-medium text-[var(--color-neutral-800)]">Git</span>
                <span class="text-[var(--color-neutral-400)]"> - For cloning repositories</span>
              </div>
            </li>
            <li class="flex items-start gap-3">
              <svg class="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
              </svg>
              <div>
                <span class="font-medium text-[var(--color-neutral-800)]">Bitbucket account</span>
                <span class="text-[var(--color-neutral-400)]"> - For authentication</span>
              </div>
            </li>
          </ul>
        </div>
      </section>

      <!-- Installation Methods -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4 flex items-center gap-2">
          <span class="w-8 h-8 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center text-sm">2</span>
          Installation Methods
        </h2>

        <!-- Cargo Install -->
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6 mb-4">
          <h3 class="font-semibold text-[var(--color-neutral-800)] mb-2 flex items-center gap-2">
            <span class="px-2 py-0.5 bg-[var(--color-success-light)] text-[var(--color-success)] text-xs rounded font-medium">Recommended</span>
            Using Cargo
          </h3>
          <p class="text-[var(--color-neutral-400)] text-sm mb-4">
            The easiest way to install Bitbucket CLI is via Cargo:
          </p>
          <div class="bg-[var(--color-neutral-900)] rounded-lg p-4">
            <code class="text-[var(--color-bitbucket-blue-light)] font-mono">cargo install bitbucket-cli</code>
          </div>
        </div>

        <!-- From Source -->
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
          <h3 class="font-semibold text-[var(--color-neutral-800)] mb-2">Building from Source</h3>
          <p class="text-[var(--color-neutral-400)] text-sm mb-4">
            Clone the repository and build manually:
          </p>
          <div class="bg-[var(--color-neutral-900)] rounded-lg p-4 space-y-2">
            <div><code class="text-[var(--color-neutral-100)] font-mono text-sm"># Clone the repository</code></div>
            <div><code class="text-[var(--color-bitbucket-blue-light)] font-mono">git clone https://github.com/pegasusheavy/bitbucket-cli.git</code></div>
            <div><code class="text-[var(--color-bitbucket-blue-light)] font-mono">cd bitbucket-cli</code></div>
            <div class="pt-2"><code class="text-[var(--color-neutral-100)] font-mono text-sm"># Build and install</code></div>
            <div><code class="text-[var(--color-bitbucket-blue-light)] font-mono">cargo install --path .</code></div>
          </div>
        </div>
      </section>

      <!-- Verify Installation -->
      <section class="mb-12">
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4 flex items-center gap-2">
          <span class="w-8 h-8 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center text-sm">3</span>
          Verify Installation
        </h2>
        <div class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6">
          <p class="text-[var(--color-neutral-400)] text-sm mb-4">
            Verify the installation by running:
          </p>
          <div class="bg-[var(--color-neutral-900)] rounded-lg p-4 mb-4">
            <code class="text-[var(--color-bitbucket-blue-light)] font-mono">bitbucket --version</code>
          </div>
          <p class="text-[var(--color-neutral-400)] text-sm">
            You should see output like: <code class="bg-[var(--color-neutral-20)] px-2 py-0.5 rounded text-[var(--color-neutral-700)]">bitbucket 1.0.0</code>
          </p>
        </div>
      </section>

      <!-- Next Steps -->
      <section>
        <h2 class="text-xl font-semibold text-[var(--color-neutral-800)] mb-4">Next Steps</h2>
        <div class="grid md:grid-cols-2 gap-4">
          <a routerLink="/authentication" class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6 hover:shadow-lg transition-shadow group">
            <div class="flex items-center gap-4">
              <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center text-2xl">
                🔐
              </div>
              <div>
                <h3 class="font-semibold text-[var(--color-neutral-800)] group-hover:text-[var(--color-bitbucket-blue)]">Authentication</h3>
                <p class="text-sm text-[var(--color-neutral-400)]">Set up your credentials</p>
              </div>
            </div>
          </a>
          <a routerLink="/configuration" class="bg-white rounded-xl border border-[var(--color-neutral-30)] p-6 hover:shadow-lg transition-shadow group">
            <div class="flex items-center gap-4">
              <div class="w-12 h-12 bg-[var(--color-bitbucket-blue-50)] rounded-lg flex items-center justify-center text-2xl">
                ⚙️
              </div>
              <div>
                <h3 class="font-semibold text-[var(--color-neutral-800)] group-hover:text-[var(--color-bitbucket-blue)]">Configuration</h3>
                <p class="text-sm text-[var(--color-neutral-400)]">Customize your settings</p>
              </div>
            </div>
          </a>
        </div>
      </section>
    </div>
  `
})
export class InstallationComponent {}
