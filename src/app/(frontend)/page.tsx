import AppWrapper from "@/components/AppWrapper";
import { Flex, Button, Text, Container, Box } from "@radix-ui/themes";
import Link from "next/link";
import { SiGithub, IconType } from "@icons-pack/react-simple-icons";
import { SiLinkedin } from "react-icons/si";

function SocialLink({
  href,
  icon: IconComponent,
  children,
}: {
  href: string;
  icon: React.ElementType;
  children: React.ReactNode;
}) {
  return (
    <Link href={href} className="border-b border-(--gray-7)">
      <Flex align="center" className="gap-x-1.5">
        <IconComponent className="size-5 text-(--gray-11)" />
        <Text className="text-(--gray-12)">{children}</Text>
      </Flex>
    </Link>
  );
}

export default async function HomePage() {
  return (
    <AppWrapper
      className="overflow-x-hidden font-schibsted"
      dotsClassName="animate-bg"
    >
      <Flex
        direction="row"
        justify="between"
        align="center"
        width="100%"
        pt="5"
        px="6"
        pb="9"
      >
        <Flex direction="column">
          <Text size="6" weight="bold" highContrast>
            Ryan Walters,
          </Text>
          <Text
            size="6"
            weight="regular"
            style={{
              color: "var(--gray-11)",
            }}
          >
            Software Engineer
          </Text>
        </Flex>

        <Text size="6" className="font-semibold" highContrast>
          About
        </Text>
      </Flex>
      <Flex align="center" direction="column">
        <Box className="max-w-2xl mx-6 border-y border-(--gray-7) divide-y divide-(--gray-7)">
          <Box py="4" className="text-(--gray-12)">
            <Text style={{ fontSize: "0.95em" }}>
              A fanatical software engineer with expertise and passion for
              sound, scalable and high-performance applications. I'm always
              working on something new. <br />
              Sometimes innovative &mdash; sometimes crazy.
            </Text>
          </Box>
          <Box py="3">
            <Text>Find me on</Text>
            <Flex gapX="2" pl="3" pt="3" pb="2">
              <SocialLink href="https://github.com/ryanwalters" icon={SiGithub}>
                GitHub
              </SocialLink>
              <SocialLink
                href="https://linkedin.com/in/ryancwalters"
                icon={SiLinkedin}
              >
                LinkedIn
              </SocialLink>
            </Flex>
          </Box>
        </Box>
      </Flex>
    </AppWrapper>
  );
}
