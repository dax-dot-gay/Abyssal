{
    pkgs,
    lib,
    config,
    inputs,
    ...
}:

{
    packages = [
        pkgs.git
        pkgs.sea-orm-cli
        pkgs.zellij
        pkgs.cargo-autoinherit
    ];
    languages = {
        rust.enable = true;
        typescript.enable = true;
        javascript = {
            enable = true;
            pnpm = {
                enable = true;
                install.enable = false;
            };
        };
    };

    services.postgres = {
        enable = true;
        listen_addresses = "127.0.0.1";
        initialDatabases = [
            {
                name = "abyssal";
                user = "abyssal";
                pass = "abyssal";
            }
        ];
        port = 5432;
    };

    scripts.dev.exec = ''
        zellij -l run.kdl
    '';

    scripts.init.exec = ''
        devenv tasks run abyssal
    '';

    tasks = {
        "abyssal:pnpm-install" = {
            exec = "pnpm install";
            cwd = "client/abyssal";
        };
        "abyssal:cargo-check" = {
            exec = "cargo check";
            cwd = "crates/abyssal";
        };
        "abyssal:certs" = {
            exec = "bash certs.sh";
            cwd = ".";
        };
    };

    env = {
        DATABASE_URL = "postgresql://abyssal:abyssal@localhost:5432/abyssal";
    };
}
