import {
    PasswordInputProps,
    PasswordInput as MantinePasswordInput,
} from "@mantine/core";
import { TbEye, TbEyeClosed } from "react-icons/tb";

export function PasswordInput(
    props: Omit<PasswordInputProps, "visibilityToggleIcon">,
) {
    return (
        <MantinePasswordInput
            {...props}
            visibilityToggleIcon={({ reveal }) =>
                reveal ? <TbEyeClosed size={18} /> : <TbEye size={18} />
            }
        />
    );
}
