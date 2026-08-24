import { createRouter, createWebHistory } from 'vue-router';
import Layout from '@/components/Layout.vue';
import Dashboard from '@/views/Dashboard.vue';
import Servers from '@/views/Servers.vue';
import Proxies from '@/views/Proxies.vue';
import Versions from '@/views/Versions.vue';
import Logs from '@/views/Logs.vue';
import Settings from '@/views/Settings.vue';
import About from '@/views/About.vue';

const routes = [
  {
    path: '/',
    component: Layout,
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: Dashboard,
      },
      {
        path: 'servers',
        name: 'Servers',
        component: Servers,
      },
      {
        path: 'proxies',
        name: 'Proxies',
        component: Proxies,
      },
      {
        path: 'versions',
        name: 'Versions',
        component: Versions,
      },
      {
        path: 'logs',
        name: 'Logs',
        component: Logs,
      },
      {
        path: 'settings',
        name: 'Settings',
        component: Settings,
      },
      {
        path: 'about',
        name: 'About',
        component: About,
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
