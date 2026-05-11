import { RecipeAppCenter } from "../features/recipes/RecipeAppCenter";

// Inside your App Center page:
<RecipeAppCenter onOpenApp={(appId) => navigate(`/apps/${appId}`)} />
