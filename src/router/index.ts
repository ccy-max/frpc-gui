import { createRouter, createWebHistory } from 'vue-router';
import Layout from '@/components/Layout.vue';

// 路由懒加载：各 view 拆独立 chunk，首屏只加载 Layout + 目标页
// （历史性能问题：7 个页面全部静态 import，1.6MB JS 全进首屏）
const Dashboard = () => import('@/views/Dashboard.vue');
const Servers = () => import('@/views/Servers.vue');
const Proxies = () => import('@/views/Proxies.vue');
const Versions = () => import('@/views/Versions.vue');
const Logs = () => import('@/views/Logs.vue');
const Settings = () => import('@/views/Settings.vue');
const About = () => import('@/views/About.vue');
const Stats = () => import('@/views/Stats.vue');
const Inspector = () => import('@/views/Inspector.vue');

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
        path: 'stats',
        name: 'Stats',
        component: Stats,
      },
      {
        path: 'inspector',
        name: 'Inspector',
        component: Inspector,
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
