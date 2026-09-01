{
  postgresql,
  postgresqlBuildExtension,
  fetchFromGitHub,
  cmake,
  ninja,
}:
   postgresqlBuildExtension (finalAttrs: {
    name = "pgu128";
    version = "1.2.0";
    src = fetchFromGitHub {
      owner = "pg-uint";
      repo = "pg-uint128";
      tag = "1.2.0";
      hash = "sha256-D2eTxzVZCQk9Cz4bhSmKu9hfL8r1RwkjS8ajadKb5sY=";
    };
  })
