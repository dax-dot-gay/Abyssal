import { SVGProps } from "react";
import LogoBlack from "../assets/logo.black.svg?react";
import LogoWhite from "../assets/logo.white.svg?react";
import { useComputedColorScheme } from "@mantine/core";

export type LogoProps = SVGProps<SVGSVGElement> & {
    title?: string;
    titleId?: string;
    desc?: string;
    descId?: string;
};

export function Logo(props: Partial<LogoProps>) {
    const scheme = useComputedColorScheme("dark");
    return scheme === "dark" ? (
        <LogoWhite {...props} />
    ) : (
        <LogoBlack {...props} />
    );
}
