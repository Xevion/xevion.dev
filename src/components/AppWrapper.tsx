import { cn } from "@/utils/helpers";
import dynamic from "next/dynamic";
import type { FunctionComponent, ReactNode } from "react";

type WrapperProps = {
  className?: string;
  dotsClassName?: string;
  children?: ReactNode;
};

const DotsDynamic = dynamic(
  () => import('@/components/Dots'),
  { ssr: false }
)

const AppWrapper: FunctionComponent<WrapperProps> = ({
  children,
  className,
  dotsClassName,
}: WrapperProps) => {
  return (
    <main
      className={cn(
        "min-h-screen text-zinc-50",
        className,
      )}
    >
      <DotsDynamic className={dotsClassName} />
      {children}
    </main>
  );
};

export default AppWrapper;
