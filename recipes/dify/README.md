# Dify Recipe

Runtime: `external-compose`

Do **not** copy Dify's full compose file into OpenNest as a manually maintained file. Dify's compose setup is large and changes with releases.

OpenNest should:

```text
clone official Dify repo
cd docker
copy .env.example to .env
run docker compose up -d
open dashboard
```

This proves OpenNest can integrate heavyweight open-source systems without owning their full deployment template.
