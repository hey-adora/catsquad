{
  postgresql,
  postgresqlBuildExtension,
  fetchFromGitHub,
  cmake,
  ninja,
  openssl,
  curl,
}:
   postgresqlBuildExtension (finalAttrs: {
    name = "pguint";
    version = "1.20260630";
    # nativeBuildInputs = [cmake ninja];
    # buildInputs = [openssl curl];
    # patches = [./pg_duckdb-httpfs-local.patch];
    # postPatch = ''
    #   substituteInPlace third_party/pg_duckdb_extensions.cmake \
    #     --subst-var-by httpfs ${duckdb-httpfs}
    # '';
    # dontConfigure = true;
    # dontBuild = true;
    # dontCheck = true;
    # installPhase = ''
    #   make install DESTDIR=$PWD
    #   mkdir $out
    #   mv $PWD${postgresql}/* $out
    # '';
    src = fetchFromGitHub {
      owner = "petere";
      repo = "pguint";
      tag = "1.20260630";
      hash = "sha256-DgGHPxui9MdvdgvdI5ES1HKsQP3hWepoaLeBWZbf9Qc=";
      # fetchSubmodules = true;
      # leaveDotGit = true;
    };
  })
        # pgunit = pgdev.postgresqlBuildExtension (finalAttrs: {
        #   pname = "pguint";
        #   version = "1.20260630";

        #   src = pkgs.fetchFromGitHub {
        #     owner = "petere";
        #     repo = "pguint";
        #     tag = "v${finalAttrs.version}";
        #     hash = "sha256-4PVr0dW6CL3ov1W5BPJU1CAphwOyXwqUoYgWCPXjto8=";
        #   };

        #   meta = {
        #     description = "unsigned integer types extension for PostgreSQL";
        #     homepage = "https://github.com/petere/pguint";
        #     license = pkgs.licenses.postgresql;
        #     platforms = pkgs.postgresql.meta.platforms;
        #     maintainers = [ ];
        #   };
        # });
