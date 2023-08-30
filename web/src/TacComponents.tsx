// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2022 Pengutronix e.K.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

import { useEffect, useState, useRef } from "react";

import Alert from "@cloudscape-design/components/alert";
import Box from "@cloudscape-design/components/box";
import Cards from "@cloudscape-design/components/cards";
import Checkbox from "@cloudscape-design/components/checkbox";
import ColumnLayout from "@cloudscape-design/components/column-layout";
import Container from "@cloudscape-design/components/container";
import Form from "@cloudscape-design/components/form";
import Header from "@cloudscape-design/components/header";
import ProgressBar from "@cloudscape-design/components/progress-bar";
import SpaceBetween from "@cloudscape-design/components/space-between";
import Spinner from "@cloudscape-design/components/spinner";
import Table from "@cloudscape-design/components/table";

import { MqttButton } from "./MqttComponents";
import { useMqttSubscription } from "./mqtt";

type RootfsSlot = {
  activated_count: string;
  activated_timestamp: string;
  bootname: string;
  boot_status: string;
  bundle_build: string;
  bundle_compatible: string;
  bundle_description: string;
  bundle_version: string;
  device: string;
  fs_type: string;
  installed_count: string;
  installed_timestamp: string;
  name: string;
  sha256: string;
  size: string;
  slot_class: string;
  state: string;
  status: string;
};

type BootloaderSlot = {
  bundle_build: string;
  bundle_compatible: string;
  bundle_description: string;
  bundle_version: string;
  device: string;
  fs_type: string;
  installed_count: string;
  installed_timestamp: string;
  name: string;
  sha256: string;
  size: string;
  state: string;
  status: string;
  slot_class: string;
};

type RaucSlots = {
  rootfs_0: RootfsSlot;
  rootfs_1: RootfsSlot;
  bootloader_0: BootloaderSlot;
};

type RaucProgress = {
  percentage: number;
  message: string;
  nesting_depth: number;
};

enum RaucInstallStep {
  Idle,
  Installing,
  Done,
}

type Duration = {
  secs: number;
  nanos: number;
};

type UpstreamBundle = {
  compatible: string;
  version: string;
  newer_than_installed: boolean;
};

type Channel = {
  name: string;
  display_name: string;
  description: string;
  url: string;
  polling_interval?: Duration;
  enabled: boolean;
  bundle?: UpstreamBundle;
};

export function SlotStatus() {
  const slot_status = useMqttSubscription<RaucSlots>("/v1/tac/update/slots");

  if (slot_status === undefined) {
    return <Spinner />;
  } else {
    let booted_slot = [];

    if (slot_status.rootfs_0.state === "booted") {
      booted_slot.push(slot_status.rootfs_0);
    }

    if (slot_status.rootfs_1.state === "booted") {
      booted_slot.push(slot_status.rootfs_1);
    }

    return (
      <SpaceBetween size="m">
        <Container
          header={
            <Header
              variant="h3"
              description="The root file system contains your applications and settings"
            >
              Root Filesystem Slots
            </Header>
          }
        >
          <Cards
            selectedItems={booted_slot}
            cardDefinition={{
              header: (e) => (typeof e === "string" ? e : e.bootname),
              sections: [
                {
                  id: "status",
                  header: "Status",
                  content: (e) => e.status,
                },
                {
                  id: "boot_status",
                  header: "Boot Status",
                  content: (e) => e.boot_status,
                },
                {
                  id: "build_date",
                  header: "Build Date",
                  content: (e) => e.bundle_build,
                },
                {
                  id: "install_date",
                  header: "Installation Date",
                  content: (e) => e.installed_timestamp,
                },
              ],
            }}
            cardsPerRow={[{ cards: 1 }, { minWidth: 500, cards: 2 }]}
            items={[slot_status.rootfs_0, slot_status.rootfs_1]}
            loadingText="Loading resources"
            selectionType="single"
            trackBy="name"
          />
        </Container>

        <Container
          header={
            <Header
              variant="h3"
              description="The bootloader is responsible for loading the Linux kernel"
            >
              Bootloader Slot
            </Header>
          }
        >
          <ColumnLayout columns={3} variant="text-grid">
            <Box>
              <Box variant="awsui-key-label">Status</Box>
              <Box>{slot_status.bootloader_0.status}</Box>
            </Box>
            <Box>
              <Box variant="awsui-key-label">Build Date</Box>
              <Box>{slot_status.bootloader_0.bundle_build}</Box>
            </Box>
            <Box>
              <Box variant="awsui-key-label">Installation Date</Box>
              <Box>{slot_status.bootloader_0.installed_timestamp}</Box>
            </Box>
          </ColumnLayout>
        </Container>
      </SpaceBetween>
    );
  }
}

export function UpdateChannels() {
  const channels_topic = useMqttSubscription<Array<Channel>>(
    "/v1/tac/update/channels",
  );

  const channels = channels_topic !== undefined ? channels_topic : [];

  return (
    <Table
      header={
        <Header
          variant="h3"
          description="Enabled update channels are periodically checked for updates"
        >
          Update Channels
        </Header>
      }
      footer={
        <Form
          actions={
            <MqttButton
              iconName="refresh"
              topic="/v1/tac/update/channels/reload"
              send={true}
            >
              Reload
            </MqttButton>
          }
        />
      }
      columnDefinitions={[
        {
          id: "name",
          header: "Name",
          cell: (e) => e.display_name,
        },
        {
          id: "enabled",
          header: "Enabled",
          cell: (e) => <Checkbox checked={e.enabled} />,
        },
        {
          id: "description",
          header: "Description",
          cell: (e) => (
            <SpaceBetween size="xs">
              {e.description.split("\n").map((p) => (
                <span>{p}</span>
              ))}
            </SpaceBetween>
          ),
        },
        {
          id: "interval",
          header: "Update Interval",
          cell: (e) => {
            if (!e.polling_interval) {
              return "Never";
            }

            let seconds = e.polling_interval.secs;
            let minutes = seconds / 60;
            let hours = minutes / 60;
            let days = hours / 24;

            if (Math.floor(days) === days) {
              return days === 1 ? "Daily" : `Every ${days} Days`;
            }

            if (Math.floor(hours) === hours) {
              return hours === 1 ? "Hourly" : `Every ${hours} Hours`;
            }

            if (Math.floor(days) === days) {
              return minutes === 1
                ? "Once a minute"
                : `Every ${minutes} Minutes`;
            }

            return `Every ${seconds} Seconds`;
          },
        },
        {
          id: "upgrade",
          header: "Upgrade",
          cell: (e) => {
            if (!e.enabled) {
              return "Not enabled";
            }

            if (!e.bundle) {
              return <Spinner />;
            }

            if (!e.bundle.newer_than_installed) {
              return "Up to date";
            }

            return (
              <MqttButton
                iconName="download"
                topic="/v1/tac/update/install"
                send={e.url}
              >
                Upgrade
              </MqttButton>
            );
          },
        },
      ]}
      items={channels}
      sortingDisabled
      trackBy="name"
    />
  );
}

export function ProgressNotification() {
  const operation = useMqttSubscription<string>("/v1/tac/update/operation");
  const progress = useMqttSubscription<RaucProgress>("/v1/tac/update/progress");
  const last_error = useMqttSubscription<string>("/v1/tac/update/last_error");

  const [installStep, setInstallStep] = useState(RaucInstallStep.Idle);
  const prev_operation = useRef<string | undefined>(undefined);

  useEffect(() => {
    if (prev_operation.current === "idle" && operation === "installing") {
      setInstallStep(RaucInstallStep.Installing);
    }

    if (prev_operation.current === "installing" && operation === "idle") {
      setInstallStep(RaucInstallStep.Done);
    }

    prev_operation.current = operation;
  }, [operation]);

  let inner = null;

  if (installStep === RaucInstallStep.Installing) {
    let valid = progress !== undefined;
    let value = progress === undefined ? 0 : progress.percentage;
    let message = progress === undefined ? "" : progress.message;

    inner = (
      <ProgressBar
        status={valid ? "in-progress" : "error"}
        value={value}
        description="Installation may take several minutes"
        additionalInfo={message}
      />
    );
  }

  if (installStep === RaucInstallStep.Done) {
    if (last_error !== undefined && last_error !== "") {
      inner = (
        <ProgressBar
          status={"error"}
          value={100}
          description="Failure"
          additionalInfo="Bundle installation failed"
        />
      );
    }
  }

  return (
    <Alert
      statusIconAriaLabel="Info"
      header="Installing Operating System Update"
      visible={inner !== null}
    >
      {inner}
    </Alert>
  );
}

export function RebootNotification() {
  const should_reboot = useMqttSubscription<boolean>(
    "/v1/tac/update/should_reboot",
  );

  return (
    <Alert
      statusIconAriaLabel="Info"
      visible={should_reboot === true}
      action={
        <MqttButton iconName="refresh" topic="/v1/tac/reboot" send={true}>
          Reboot
        </MqttButton>
      }
      header="Reboot into other slot"
    >
      There is a newer operating system bundle installed in the other boot slot.
      Reboot now to use it.
    </Alert>
  );
}

export function UpdateContainer() {
  return (
    <Container
      header={
        <Header
          variant="h2"
          description="Check your redundant update status and slots"
        >
          RAUC
        </Header>
      }
    >
      <SpaceBetween size="m">
        <UpdateChannels />
        <SlotStatus />
      </SpaceBetween>
    </Container>
  );
}

export function UpdateNotification() {
  const channels = useMqttSubscription<Array<Channel>>(
    "/v1/tac/update/channels",
  );

  let updates = [];

  if (channels !== undefined) {
    for (let ch of channels) {
      if (ch.enabled && ch.bundle && ch.bundle.newer_than_installed) {
        updates.push(ch);
      }
    }
  }

  const install_buttons = updates.map((u) => (
    <MqttButton
      key={u.name}
      iconName="download"
      topic="/v1/tac/update/install"
      send={u.url}
    >
      Install new {u.display_name} bundle
    </MqttButton>
  ));

  let text =
    "There is a new operating system update available for installation";

  if (updates.length > 1) {
    text =
      "There are new operating system updates available available for installation";
  }

  return (
    <Alert
      statusIconAriaLabel="Info"
      visible={updates.length > 0}
      action={<SpaceBetween size="xs">{install_buttons}</SpaceBetween>}
      header="Update your LXA TAC"
    >
      {text}
    </Alert>
  );
}

export function LocatorNotification() {
  const locator = useMqttSubscription<boolean>("/v1/tac/display/locator");

  return (
    <Alert
      statusIconAriaLabel="Info"
      visible={locator === true}
      action={
        <MqttButton topic="/v1/tac/display/locator" send={false}>
          Found it!
        </MqttButton>
      }
      header="Find this TAC"
    >
      Someone is looking for this TAC.
    </Alert>
  );
}

export function OverTemperatureNotification() {
  const warning = useMqttSubscription<string>("/v1/tac/temperatures/warning");

  return (
    <Alert
      statusIconAriaLabel="Warning"
      type="warning"
      visible={warning !== "Okay"}
      header="Your LXA TAC is overheating"
    >
      The LXA TAC's temperature is{" "}
      {warning === "SocCritical" ? "critical" : "high"}. Provide better airflow
      and check for overloads!
    </Alert>
  );
}
