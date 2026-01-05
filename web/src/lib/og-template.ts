/**
 * Generate OG image HTML template matching xevion.dev dark aesthetic.
 * Satori only supports flex layouts and subset of CSS.
 */
export function generateOGTemplate({
  title,
  subtitle,
  type = "default",
}: {
  title: string;
  subtitle?: string;
  type?: "default" | "project";
}): string {
  return `
    <div
      style="
        display: flex;
        width: 1200px;
        height: 630px;
        background-color: #000000;
        color: #fafafa;
        font-family: 'Schibsted Grotesk', sans-serif;
        padding: 60px 80px;
      "
    >
      <div
        style="
          display: flex;
          flex-direction: column;
          justify-content: space-between;
          width: 100%;
          height: 100%;
        "
      >
        <!-- Main Content -->
        <div style="display: flex; flex-direction: column; flex: 1; justify-content: center;">
          <h1
            style="
              font-family: 'Hanken Grotesk', sans-serif;
              font-weight: 900;
              font-size: ${type === "project" ? "72px" : "96px"};
              line-height: 1.1;
              margin: 0;
              color: #ffffff;
            "
          >
            ${escapeHtml(title)}
          </h1>
          ${
            subtitle
              ? `
          <p
            style="
              font-family: 'Schibsted Grotesk', sans-serif;
              font-size: 36px;
              margin: 32px 0 0 0;
              color: #a1a1aa;
              line-height: 1.4;
            "
          >
            ${escapeHtml(subtitle)}
          </p>
          `
              : ""
          }
        </div>

        <!-- Footer -->
        <div
          style="
            display: flex;
            justify-content: space-between;
            align-items: flex-end;
            border-top: 2px solid #27272a;
            padding-top: 24px;
          "
        >
          <div
            style="
              font-size: 28px;
              color: #71717a;
              font-weight: 500;
            "
          >
            xevion.dev
          </div>
          ${
            type === "project"
              ? `
          <div
            style="
              font-size: 24px;
              color: #52525b;
              text-transform: uppercase;
              letter-spacing: 0.05em;
            "
          >
            PROJECT
          </div>
          `
              : ""
          }
        </div>
      </div>
    </div>
  `;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
