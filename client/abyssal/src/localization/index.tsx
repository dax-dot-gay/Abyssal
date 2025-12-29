import { createInstance } from "i18next";
import { I18nextProvider } from "react-i18next";

import * as lang_en from "./lang/en.json";
import { ReactNode } from "react";

const i18n = createInstance({
    fallbackLng: "en",
    interpolation: {
        escapeValue: false,
    },
    resources: {
        en: {
            translation: lang_en,
        },
    },
});

i18n.init();

export function LocalizationProvider({
    children,
}: {
    children?: ReactNode | ReactNode[];
}) {
    return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>;
}
