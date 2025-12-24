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
    ];
    languages = {
        rust.enable = true;
        typescript.enable = true;
        javascript = {
            enable = true;
            pnpm = {
                enable = true;
                install.enable = true;
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

    env = {
        ABYSSAL_DB__BACKEND = "postgres";
        ABYSSAL_DB__URL = "postgresql://abyssal:abyssal@localhost:5432/abyssal";
        DATABASE_URL = config.env.ABYSSAL_DB__URL;
    };
}
