import { useState } from "react";

import Box from "@cloudscape-design/components/box";
import Button from "@cloudscape-design/components/button";
import Cards from "@cloudscape-design/components/cards";
import Container from "@cloudscape-design/components/container";
import Header from "@cloudscape-design/components/header";
import Form from "@cloudscape-design/components/form";
import SpaceBetween from "@cloudscape-design/components/space-between";
import Spinner from "@cloudscape-design/components/spinner";
import Wizard from "@cloudscape-design/components/wizard";

import { ConfigEditor, LabgridService, LabgridConfig } from "./SettingsLabgrid";
import { useMqttState } from "./mqtt";

const SSH_AUTH_KEYS_EXAMPLE =
  "\
# Paste one (or multiple) of your ssh public keys here.\n\
# Use 'cat ~/.ssh/id_*.pub' to get a list of your ssh public\n\
# keys or ssh-keygen if you don't have any yet.\n\
# They will look something like this:\n\
# ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBlPtT5dnGcZn0Z6FyD6VGqt3Jx0s+BHhMahxR0KlJ8G tux@igloo\n\
";

enum WizardChoice {
  Undecided,
  Ssh,
  CustomBundle,
}

interface WizardProps {
  setWizard: (w: WizardChoice) => void;
  setSetupMode: (m: boolean) => void;
}

function WizardSelector(props: WizardProps) {
  const [selection, setSelection] = useState<any>();
  const [canContinue, setCanContinue] = useState(false);

  return (
    <Container
      header={
        <Header
          variant="h1"
          description="Choose how you want to continue with the setup of your LXA TAC"
        >
          Setup Wizard
        </Header>
      }
    >
      <SpaceBetween size="xxl">
        <Box variant="p" fontSize="display-l" textAlign="center">
          Welcome to your LXA TAC!
        </Box>
        <Box variant="p">
          The following wizard will guide you through the first time setup of
          your TAC. Begin the setup process by choosing whether to keep using
          the software that is already installed on your TAC or by installing a
          pre-configured RAUC bundle you have prepared.
        </Box>
        <Form
          actions={
            <Button
              variant="primary"
              disabled={!canContinue}
              onClick={(d) => props.setWizard(selection[0].kind)}
            >
              Continue
            </Button>
          }
        >
          <Cards
            onSelectionChange={({ detail }) => {
              setCanContinue(true);
              setSelection(detail.selectedItems);
            }}
            selectedItems={selection}
            cardDefinition={{
              header: (e) => e.name,
              sections: [
                {
                  id: "benefits",
                  header: "Benefits:",
                  content: (e) => e.benefits,
                },
                {
                  id: "drawbacks",
                  header: "Drawbacks:",
                  content: (e) => e.drawbacks,
                },
              ],
            }}
            cardsPerRow={[{ cards: 2 }]}
            items={[
              {
                name: "Setup via SSH keys",
                kind: WizardChoice.Ssh,
                benefits: (
                  <ul>
                    <li>Get started immediately</li>
                    <li>Always get the newest software for your LXA TAC</li>
                    <li>Easier to set up</li>
                  </ul>
                ),
                drawbacks: (
                  <ul>
                    <li>Can get tedious in larger fleets</li>
                  </ul>
                ),
              },
              {
                name: "Setup via custom RAUC bundle",
                kind: WizardChoice.CustomBundle,
                benefits: (
                  <ul>
                    <li>
                      Quickly integrate new LXA TACs into an existing fleet
                    </li>
                    <li>Deploy a custom selection of software</li>
                    <li>Deploy custom config</li>
                  </ul>
                ),
                drawbacks: (
                  <ul>
                    <li>
                      Requires up-front work to configure and build bundles
                    </li>
                    <li>
                      You have to manually re-build bundles to get software
                      updates
                    </li>
                  </ul>
                ),
              },
            ]}
            selectionType="single"
            trackBy="name"
          />
        </Form>
      </SpaceBetween>
    </Container>
  );
}

function SshWizard(props: WizardProps) {
  const [activeStepIndex, setActiveStepIndex] = useState(0);

  return (
    <Container
      header={
        <Header variant="h1" description="Setup via SSH keys">
          SSH key based Wizard
        </Header>
      }
    >
      <Wizard
        i18nStrings={{
          stepNumberLabel: (stepNumber) => `Step ${stepNumber}`,
          collapsedStepsLabel: (stepNumber, stepsCount: number) =>
            `Step ${stepNumber} of ${stepsCount}`,
          skipToButtonLabel: (step, stepNumber) => `Skip to ${step.title}`,
          navigationAriaLabel: "Steps",
          cancelButton: "Cancel",
          previousButton: "Previous",
          nextButton: "Next",
          submitButton: "Done",
          optional: "optional",
        }}
        onCancel={() => props.setWizard(WizardChoice.Undecided)}
        onSubmit={() => props.setSetupMode(false)}
        onNavigate={({ detail }) =>
          setActiveStepIndex(detail.requestedStepIndex)
        }
        activeStepIndex={activeStepIndex}
        allowSkipTo
        steps={[
          {
            title: "Add SSH keys",
            description:
              "Deploy SSH keys on your LXA TAC so you can log into it",
            content: (
              <Container>
                <ConfigEditor
                  path="/v1/tac/ssh/authorized_keys"
                  language="text"
                  defaultContent={SSH_AUTH_KEYS_EXAMPLE}
                />
              </Container>
            ),
          },
          {
            title: "Configure Labgrid",
            description: "Configure your Labgrid Exporter",
            isOptional: true,
            content: (
              <Container>
                <LabgridConfig />
              </Container>
            ),
          },
          {
            title: "Test Labgrid",
            description: "Make sure your Labgrid Exporter Service looks healty",
            isOptional: true,
            content: (
              <Container>
                <LabgridService />
              </Container>
            ),
          },
          {
            title: "Complete Setup",
            description: "Make sure everything is working alright",
            content: (
              <Container>
                <Box>You are about to complete the setup wizard.</Box>
                <Box>
                  You will not be able to re-enter the setup wizard to deploy
                  new SSH keys via the web interface. You can however re-enable
                  the setup wizard using the buttons and the screen on the
                  device. If you do not have physical access the TAC you should
                  make sure that you can log in to the device via the SSH keys
                  you have deployed before pressing "Done".
                </Box>
              </Container>
            ),
          },
        ]}
      />
    </Container>
  );
}

function CustomBundleWizard() {
  return (
    <Container
      header={
        <Header variant="h1" description="Setup via custom RAUC bundle">
          RAUC based Wizard
        </Header>
      }
    >
      <Box>Is not yet implemented, sorry</Box>
    </Container>
  );
}

function SetupComplete() {
  return (
    <Container
      header={
        <Header variant="h1" description="Your LXA TAC is already set up">
          Setup Complete
        </Header>
      }
    >
      <Form
        actions={
          <Button variant="link" href="/#/">
            Start Exploring!
          </Button>
        }
      >
        <SpaceBetween size="m">
          <Box>
            It looks like your LXA TAC is fully set up and you are ready to
            explore its features 🎉!
          </Box>
          <Box>
            You can always go back to the setup mode by going to the system
            screen on the on-device LCD and selecting the setup mode.
          </Box>
        </SpaceBetween>
      </Form>
    </Container>
  );
}

export default function Setup() {
  const [setupModeSettled, setupMode, setSetupMode] =
    useMqttState<boolean>("/v1/tac/setup_mode");
  const [wizard, setWizard] = useState(WizardChoice.Undecided);

  let content = undefined;

  if (setupMode === undefined || !setupModeSettled) {
    content = (
      <div id="setup_wizard_spinner">
        <Spinner size="large" />
      </div>
    );
  } else if (setupMode) {
    switch (wizard) {
      case WizardChoice.Ssh: {
        content = (
          <SshWizard setWizard={setWizard} setSetupMode={setSetupMode} />
        );
        break;
      }
      case WizardChoice.CustomBundle: {
        content = <CustomBundleWizard />;
        break;
      }
      default: {
        content = (
          <WizardSelector setWizard={setWizard} setSetupMode={setSetupMode} />
        );
        break;
      }
    }
  } else {
    content = <SetupComplete />;
  }

  return (
    <div id="setup_wizard_outer">
      <div id="setup_wizard_inner">{content}</div>
    </div>
  );
}
