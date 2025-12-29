import { MantineProvider } from "@mantine/core";
import { LocalizationProvider } from "./localization";
import { shadcnTheme } from "./util/theme/theme";
import { shadcnCssVariableResolver } from "./util/theme/resolver";

export function AbyssalRoot() {
    return (
        <LocalizationProvider>
            <MantineProvider
                theme={shadcnTheme}
                cssVariablesResolver={shadcnCssVariableResolver}
                defaultColorScheme="dark"
            ></MantineProvider>
        </LocalizationProvider>
    );
}
