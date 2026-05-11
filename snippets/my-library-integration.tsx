import { MyLibraryRecipes } from "../features/recipes/MyLibraryRecipes";

// Inside your My Library page:
<MyLibraryRecipes onOpenDetails={(appId) => navigate(`/apps/${appId}`)} />
