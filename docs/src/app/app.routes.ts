import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    loadComponent: () => import('./pages/home/home.component').then(m => m.HomeComponent)
  },
  {
    path: 'installation',
    loadComponent: () => import('./pages/installation/installation.component').then(m => m.InstallationComponent)
  },
  {
    path: 'authentication',
    loadComponent: () => import('./pages/authentication/authentication.component').then(m => m.AuthenticationComponent)
  },
  {
    path: 'configuration',
    loadComponent: () => import('./pages/configuration/configuration.component').then(m => m.ConfigurationComponent)
  },
  {
    path: 'tui',
    loadComponent: () => import('./pages/tui/tui.component').then(m => m.TuiComponent)
  },
  {
    path: 'commands/auth',
    loadComponent: () => import('./pages/commands/auth/auth.component').then(m => m.AuthCommandComponent)
  },
  {
    path: 'commands/repo',
    loadComponent: () => import('./pages/commands/repo/repo.component').then(m => m.RepoCommandComponent)
  },
  {
    path: 'commands/pr',
    loadComponent: () => import('./pages/commands/pr/pr.component').then(m => m.PrCommandComponent)
  },
  {
    path: 'commands/issue',
    loadComponent: () => import('./pages/commands/issue/issue.component').then(m => m.IssueCommandComponent)
  },
  {
    path: 'commands/pipeline',
    loadComponent: () => import('./pages/commands/pipeline/pipeline.component').then(m => m.PipelineCommandComponent)
  },
  {
    path: 'commands/workspace',
    loadComponent: () => import('./pages/commands/workspace/workspace.component').then(m => m.WorkspaceCommandComponent)
  },
  {
    path: 'commands/user',
    loadComponent: () => import('./pages/commands/user/user.component').then(m => m.UserCommandComponent)
  },
  {
    path: '**',
    loadComponent: () => import('./pages/not-found/not-found.component').then(m => m.NotFoundComponent)
  }
];
