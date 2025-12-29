import { MantineProvider } from "@mantine/core";
import { LocalizationProvider } from "./localization";
import { shadcnTheme } from "./util/theme/theme";
import { shadcnCssVariableResolver } from "./util/theme/resolver";
import { ApiProvider } from "./client";

export function AbyssalRoot() {
    return (
        <ApiProvider>
            <LocalizationProvider>
                <MantineProvider
                    theme={shadcnTheme}
                    cssVariablesResolver={shadcnCssVariableResolver}
                    defaultColorScheme="dark"
                ></MantineProvider>
            </LocalizationProvider>
        </ApiProvider>
    );
}
