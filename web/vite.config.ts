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
  },
})
