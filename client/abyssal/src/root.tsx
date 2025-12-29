import { MantineProvider } from "@mantine/core";
import { LocalizationProvider } from "./localization";
import { shadcnTheme } from "./util/theme/theme";
import { shadcnCssVariableResolver } from "./util/theme/resolver";
import { ApiProvider } from "./client";
import { Notifications } from "@mantine/notifications";
import { ModalsProvider } from "@mantine/modals";
import { RouterProvider } from "react-router/dom";
import { AbyssalRouter } from "./routes";

export function AbyssalRoot() {
    return (
        <ApiProvider>
            <LocalizationProvider>
                <MantineProvider
                    theme={shadcnTheme}
                    cssVariablesResolver={shadcnCssVariableResolver}
                    defaultColorScheme="dark"
                >
                    <Notifications />
                    <ModalsProvider>
                        <RouterProvider router={AbyssalRouter} />
                    </ModalsProvider>
                </MantineProvider>
            </LocalizationProvider>
        </ApiProvider>
    );
}
