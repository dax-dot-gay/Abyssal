import {
    Button,
    Center,
    Divider,
    Group,
    Paper,
    Stack,
    TextInput,
    Title,
} from "@mantine/core";
import { useTranslation } from "react-i18next";
import "./style.scss";
import { Logo } from "../../util/logo";
import { useForm } from "@mantine/form";
import { TbLock, TbLogin2, TbUser } from "react-icons/tb";
import { PasswordInput } from "../../util/password_input";
import { SDK, useApi, useToken } from "../../client";
import { useNavigate } from "react-router";
import { useEffect } from "react";
import { useNotifications } from "../../util/notifications";

export function LoginView() {
    const { t } = useTranslation();
    const token = useToken();
    const api = useApi();
    const nav = useNavigate();
    const { success, error } = useNotifications();

    useEffect(() => {
        if (token !== null) {
            nav("/");
        }
    }, [nav, token]);

    const loginForm = useForm({
        initialValues: {
            username: "",
            password: "",
        },
        validate: {
            username: (value) =>
                value.length === 0 ? t("error.form.empty") : null,
            password: (value) =>
                value.length === 0 ? t("error.form.empty") : null,
        },
    });
    return (
        <form
            onSubmit={loginForm.onSubmit(({ username, password }) => {
                SDK.login({ body: { username, password } }).then((result) => {
                    if (result.data) {
                        api.login(result.data.token, result.data.user);
                        success(
                            t("login.success", {
                                username: result.data.user.name,
                            }),
                        );
                        nav("/");
                    } else {
                        api.logout();
                        error(result.error.message, t("login.error"));
                    }
                });
            })}
        >
            <Center w="100vw" h="100vh" className="login-view root">
                <Paper
                    className="login-view wrapper"
                    p="sm"
                    withBorder
                    variant="filled"
                >
                    <Stack gap="sm">
                        <Group gap="sm" w="100%" justify="space-between">
                            <Logo width={36} height={36} />
                            <Title order={3} ff="monospace">
                                {t("app.name")}/*
                            </Title>
                        </Group>
                        <Divider />
                        <TextInput
                            {...loginForm.getInputProps("username")}
                            label={t("login.username")}
                            leftSection={<TbUser size={18} />}
                        />
                        <PasswordInput
                            {...loginForm.getInputProps("password")}
                            label={t("login.password")}
                            leftSection={<TbLock size={18} />}
                        />
                        <Button
                            size="md"
                            leftSection={<TbLogin2 size={24} />}
                            justify="space-between"
                            px="sm"
                            type="submit"
                        >
                            {t("login.submit")}
                        </Button>
                    </Stack>
                </Paper>
            </Center>
        </form>
    );
}
