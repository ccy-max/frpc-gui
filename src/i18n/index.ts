import { createI18n } from 'vue-i18n';
import { zhCN, enUS } from './locales';

// 单例 i18n：main.ts 注册插件，store 切换语言时改 locale
export const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
});

export function setLocale(lang: 'zh-CN' | 'en-US') {
  i18n.global.locale.value = lang;
  document.documentElement.setAttribute('lang', lang);
}

export default i18n;
