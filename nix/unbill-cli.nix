{
  lib,
  rustPlatform,
  fetchFromGitHub,
  src ? fetchFromGitHub {
    owner = "unbill-project";
    repo = "unbill";
    tag = "v${version}";
    hash = lib.fakeHash;
  },
  version ? "0.0.1",
}:

rustPlatform.buildRustPackage {
  pname = "unbill-cli";
  inherit version src;

  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = [ "--package" "unbill-cli" ];
  cargoTestFlags = [ "--package" "unbill-cli" ];

  meta = {
    description = "Command-line interface for unbill";
    homepage = "https://github.com/unbill-project/unbill";
    license = with lib.licenses; [ mit asl20 ];
    maintainers = with lib.maintainers; [ ];
    mainProgram = "unbill-cli";
  };
}
