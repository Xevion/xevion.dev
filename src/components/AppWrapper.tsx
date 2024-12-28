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
      {!hideNavigation ? (
        <Disclosure as="nav">
          {({ open }) => (
            <>
              <div className="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
                <div className="relative flex h-16 items-center justify-between">
                  <div className="absolute inset-y-0 left-0 flex items-center sm:hidden">
                    {/* Mobile menu button*/}
                    <Disclosure.Button className="inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-zinc-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white">
                      <span className="sr-only">Open main menu</span>
                      {open ? (
                        <HiXMark className="block h-6 w-6" aria-hidden="true" />
                      ) : (
                        <HiBars3 className="block h-6 w-6" aria-hidden="true" />
                      )}
                    </Disclosure.Button>
                  </div>
                  <div className="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
                    <div className="hidden sm:ml-6 sm:block">
                      <div className="flex space-x-4 font-roboto text-lg">
                        {navigation.map((item) => (
                          <Link
                            key={item.name}
                            href={item.href}
                            className={classNames(
                              item.id == current
                                ? "bg-zinc-900 text-white"
                                : "text-gray-300 hover:bg-zinc-800/60 hover:text-white",
                              "rounded-md px-2.5 py-1",
                            )}
                            aria-current={
                              item.id == current ? "page" : undefined
                            }
                          >
                            {item.name}
                          </Link>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <Disclosure.Panel className="sm:hidden">
                <div className="space-y-1 px-2 pb-3 pt-2">
                  {navigation.map((item) => (
                    <Link key={item.name} href={item.href}>
                      <Disclosure.Button
                        as="a"
                        className={classNames(
                          item.id == current
                            ? "bg-zinc-900 text-white"
                            : "text-gray-300 hover:bg-zinc-700 hover:text-white",
                          "block rounded-md px-3 py-2 text-base font-medium",
                        )}
                        aria-current={item.id == current ? "page" : undefined}
                      >
                        {item.name}
                      </Disclosure.Button>
                    </Link>
                  ))}
                </div>
              </Disclosure.Panel>
            </>
          )}
        </Disclosure>
      ) : null}
      <DotsDynamic />
      {children}
    </main>
  );
};

export default AppWrapper;
