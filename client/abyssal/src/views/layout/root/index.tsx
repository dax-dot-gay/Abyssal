import {
    ActionIcon,
    AppShell,
    Avatar,
    Burger,
    Button,
    Center,
    Divider,
    Group,
    Kbd,
    Paper,
    ScrollArea,
    Stack,
    Text,
    TextInput,
    Title,
    Tooltip,
} from "@mantine/core";
import { useDisclosure, useInputState } from "@mantine/hooks";
import { useTranslation } from "react-i18next";
import {
    TbFolderFilled,
    TbLogout,
    TbSearch,
    TbServerCog,
    TbSettingsFilled,
    TbShieldFilled,
    TbUser,
} from "react-icons/tb";
import { Logo } from "../../../util/logo";
import { SDK, useApi } from "../../../client";
import { useNavigate } from "react-router";
import { useEffect } from "react";
import "./style.scss";

export function AppLayout() {
    const [opened, { toggle }] = useDisclosure();
    const [pathValue, setPathValue] = useInputState("/");
    const { t } = useTranslation();
    const api = useApi();
    const nav = useNavigate();

    useEffect(() => {
        if (api.token === null) {
            nav("/auth");
        }
    }, [api.token, nav]);

    return api.token ? (
        <AppShell
            navbar={{
                width: 300,
                breakpoint: "sm",
                collapsed: { mobile: !opened },
            }}
            padding="sm"
            header={{ height: 48 }}
            id="layout"
            layout="alt"
        >
            <AppShell.Header
                withBorder={false}
                style={{ borderColor: "var(--mantine-color-secondary-filled)" }}
                id="layout-header"
            >
                <Paper w="100%" h="100%" p={0} shadow="lg" radius={0}>
                    <Group
                        gap={8}
                        w="100%"
                        h="100%"
                        align="center"
                        wrap="nowrap"
                        p={0}
                        px={8}
                    >
                        <Center hiddenFrom="sm" w={32} h={48}>
                            <Burger
                                opened={opened}
                                onClick={toggle}
                                size={24}
                                hiddenFrom="sm"
                            />
                        </Center>
                        <Divider orientation="vertical" hiddenFrom="sm" />
                        <TextInput
                            leftSection={<TbFolderFilled size={20} />}
                            size="sm"
                            ff="monospace"
                            value={pathValue}
                            onChange={setPathValue}
                            placeholder={t("layout.path")}
                            style={{ flexGrow: 2 }}
                            variant="filled"
                        />
                        <Tooltip
                            p="6"
                            label={
                                <Text ff="monospace">
                                    <Kbd>Ctrl</Kbd> + <Kbd>P</Kbd>
                                </Text>
                            }
                        >
                            <Button
                                size="sm"
                                leftSection={<TbSearch size={20} />}
                            >
                                {t("layout.search")}
                            </Button>
                        </Tooltip>
                    </Group>
                </Paper>
            </AppShell.Header>
            <AppShell.Navbar
                p={0}
                withBorder
                style={{ borderColor: "var(--mantine-color-secondary-filled)" }}
                id="layout-navbar"
            >
                <Stack gap={0} className="nav-stack">
                    <Group
                        style={{
                            borderBottom:
                                "1px solid var(--mantine-color-secondary-filled",
                        }}
                        h={48}
                        p="xs"
                        justify="space-between"
                    >
                        <Logo width={24} height={24} />
                        <Title ff="monospace" order={4}>
                            Abyssal/*
                        </Title>
                    </Group>
                    <ScrollArea className="nav-scroll" w="100%" px="sm">
                        <Stack
                            gap="sm"
                            className="nav-scroll-stack"
                            py="sm"
                        ></Stack>
                    </ScrollArea>
                    <Group
                        style={{
                            borderTop:
                                "1px solid var(--mantine-color-secondary-filled",
                        }}
                        h={60}
                        p="xs"
                        wrap="nowrap"
                        gap="xs"
                    >
                        <Avatar size="md">
                            {api.user.permissions.filter(
                                (v) => v.permission === "administrator",
                            ).length > 0 ? (
                                <TbShieldFilled size={20} />
                            ) : (
                                <TbUser size={20} />
                            )}
                        </Avatar>
                        <Stack gap={0} style={{ flexGrow: 1 }}>
                            <Text size="md">{api.user.name}</Text>
                            <Text size="xs" c="dimmed" ff="monospace">
                                {api.user.kind}
                            </Text>
                        </Stack>
                        {api.user.permissions.filter(
                            (v) => v.permission === "administrator",
                        ).length > 0 && (
                            <ActionIcon size="lg" variant="light">
                                <TbServerCog size={20} />
                            </ActionIcon>
                        )}
                        <ActionIcon size="lg" variant="light">
                            <TbSettingsFilled size={20} />
                        </ActionIcon>
                        <ActionIcon
                            size="lg"
                            variant="light"
                            onClick={() => {
                                SDK.logout();
                                api.logout();
                                nav("/");
                            }}
                        >
                            <TbLogout size={20} />
                        </ActionIcon>
                    </Group>
                </Stack>
            </AppShell.Navbar>
        </AppShell>
    ) : (
        <></>
    );
}
