import AppWrapper from "@/components/AppWrapper";
import { Flex, Button, Text, Container, Box } from "@radix-ui/themes";
import Link from "next/link";
import { SiGithub, IconType } from "@icons-pack/react-simple-icons";
import { SiLinkedin } from "react-icons/si";
import { Rss } from "lucide-react";

function NavLink({
  href,
  children,
}: {
  href: string;
  children: React.ReactNode;
}) {
  return (
    <Link href={href}>
      <Text size="3" className="text-(--gray-11) hover:text-(--gray-12)">
        {children}
      </Text>
    </Link>
  );
}

function IconLink({
  href,
  icon: Icon,
}: {
  href: string;
  icon: React.ElementType;
}) {
  return (
    <Link href={href} className="text-(--gray-11) hover:text-(--gray-12)">
      <Icon className="size-5" />
    </Link>
  );
}

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
    <Link href={href}>
      <Flex
        align="center"
        className="gap-x-1.5 px-1.5 py-1 rounded-xs bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
      >
        <IconComponent className="size-4 text-zinc-300" />
        <Text size="2" className="text-zinc-100">
          {children}
        </Text>
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
      {/* Top Navigation Bar */}
      <Flex justify="end" align="center" width="100%" pt="5" px="6" pb="9">
        <Flex gap="4" align="center">
          <NavLink href="/projects">Projects</NavLink>
          <NavLink href="/blog">Blog</NavLink>
          <IconLink href="https://github.com/Xevion" icon={SiGithub} />
          <IconLink href="/rss" icon={Rss} />
        </Flex>
      </Flex>

      {/* Main Content */}
      <Flex align="center" direction="column">
        <Box className="max-w-2xl mx-6 border-b border-(--gray-7) divide-y divide-(--gray-7)">
          {/* Name & Job Title */}
          <Flex direction="column" pb="4">
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
              <SocialLink href="https://github.com/Xevion" icon={SiGithub}>
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
