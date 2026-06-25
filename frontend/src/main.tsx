import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
// flag-icons MUST load before our own styles so theme.css can override the
// .fi sizing for the pill wrapper.
import 'flag-icons/css/flag-icons.min.css';
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
