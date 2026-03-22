/**
 * 设计系统组件 barrel export
 *
 * 纯 UI 组件，零业务逻辑。
 * 过渡期间：从 components/ 目录 re-export。
 */
export { default as Button } from "../components/Button";
export { default as Card } from "../components/Card";
export { default as Input } from "../components/Input";
export { default as Modal } from "../components/Modal";
export { default as Alert } from "../components/Alert";
export { Spinner } from "../components/Spinner";
export { default as Badge } from "../components/Badge";
export { Icon } from "../components/Icon";
export { default as ErrorBoundary } from "../components/ErrorBoundary";
