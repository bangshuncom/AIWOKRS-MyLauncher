import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";
import "./styles/form.css";
import { t } from "./i18n";

const app = createApp(App);

// 全局翻译函数：模板中可直接使用 $t('key')
// （script setup 内推荐用 useI18n() 解构，类型更友好）
app.config.globalProperties.$t = t;

app.mount("#app");
