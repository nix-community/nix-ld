let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);

  flake-compat = builtins.fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
    sha256 = lock.nodes.flake-compat.locked.narHash;
  };

  flake = import flake-compat {
    src = ./.;
  };

  shell = flake.shellNix.default // {
    reproduce = flake.defaultNix.outputs.reproduce.${builtins.currentSystem};
  };
in shell
