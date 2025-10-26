import { getPayload } from "payload";
import config from "../../../payload.config";
import { NextResponse } from "next/server";

export async function GET(req: Request) {
  const secret = req.headers.get("authorization");
  const healthcheckSecret = process.env.HEALTHCHECK_SECRET;

  if (!secret || !healthcheckSecret || secret !== healthcheckSecret) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  try {
    // Try a simple Payload API call (fetch one project)
    const payloadConfig = await config;
    const payload = await getPayload({ config: payloadConfig });

    await payload.find({
      collection: "projects",
      limit: 1,
    });

    return NextResponse.json({ status: "ok" }, { status: 200 });
  } catch (error) {
    return NextResponse.json(
      { error: "Payload CMS unhealthy", details: String(error) },
      { status: 500 },
    );
  }
}
