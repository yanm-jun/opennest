import { AppCenterPage } from "./AppCenterPage";

export function RecipeAppCenter({ onOpenApp }: { onOpenApp: (appId: string) => void }) {
  return <AppCenterPage onOpenApp={onOpenApp} />;
}
