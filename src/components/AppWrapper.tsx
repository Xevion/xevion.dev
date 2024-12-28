import { cn } from "@/utils/helpers";
import { Disclosure } from "@headlessui/react";
import dynamic from "next/dynamic";
import Link from "next/link";
import type { FunctionComponent, ReactNode } from "react";
import { HiBars3, HiXMark } from "react-icons/hi2";

const navigation: { id: string; name: string; href: string }[] = [
  { id: "home", name: "Home", href: "/" },
  { id: "projects", name: "Projects", href: "/projects" },
  { id: "contact", name: "Contact", href: "/contact" },
];

type WrapperProps = {
  className?: string;
  hideNavigation?: boolean;
  current?: string;
  children?: ReactNode | ReactNode[] | null;
};

const DotsDynamic = dynamic(
  () => import('@/components/Dots'),
  { ssr: false }
)

const AppWrapper: FunctionComponent<WrapperProps> = ({
  current,
  children,
  hideNavigation,
  className,
}: WrapperProps) => {
  return (
    <main
      className={cn(
        "min-h-screen text-zinc-50",
        className,
      )}
    >
      <DotsDynamic />
      {children}
    </main>
  );
};

export default AppWrapper;
