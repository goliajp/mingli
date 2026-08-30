import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// 端口由 port-registry 分配：mingli-web=6026，代理 /api → mingli-api=6027。
// host:true 绑所有接口(含 IPv4 127.0.0.1)，避免只绑 ::1 导致浏览器打不开。
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 6026,
    proxy: {
      '/api': 'http://127.0.0.1:6027',
    },
    // CI 的容器里 inotify 常常收不到改动，于是改了文件、页面还跑着旧模块。
    // 平时无所谓（人会自己刷新），但守卫自检要靠「改一行源码 → 页面真的变了」才成立：
    // 监视漏一次，断言就绿在「没测到」上。那一族只在 CI 上跑，故只在那里开轮询。
    watch: process.env.MINGLI_WATCH_POLL ? { usePolling: true, interval: 300 } : undefined,
  },
})
