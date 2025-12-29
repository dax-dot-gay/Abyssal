import { notifications } from "@mantine/notifications";
import { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { TbCircleCheckFilled, TbCircleXFilled } from "react-icons/tb";

export function useNotifications() {
    const { t } = useTranslation();
    return {
        success: (message?: ReactNode, title?: ReactNode) =>
            notifications.show({
                icon: <TbCircleCheckFilled size={20} />,
                title: title ?? t("notification.success"),
                message,
                color: "green",
            }),
        error: (message?: ReactNode, title?: ReactNode) =>
            notifications.show({
                icon: <TbCircleXFilled size={20} />,
                title: title ?? t("notification.error"),
                message,
                color: "red",
            }),
    };
}
