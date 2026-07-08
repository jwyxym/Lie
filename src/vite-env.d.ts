/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
};
declare module 'mui-player';

interface Window {
  LieAndroid?: {
    lockLandscape(): void;
    lockPortrait(): void;
    unlockOrientation(): void;
    hideNavigation(): void;
    showNavigation(): void;
    hideStatusBar(): void;
    showStatusBar(): void;
  };
}
