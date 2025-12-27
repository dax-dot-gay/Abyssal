{
    pkgs,
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
}
