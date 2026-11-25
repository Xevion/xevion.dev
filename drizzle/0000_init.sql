CREATE TABLE `app_state` (
	`key` text PRIMARY KEY NOT NULL,
	`value` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `device_auth_requests` (
	`id` text PRIMARY KEY NOT NULL,
	`user_code` text NOT NULL,
	`label` text,
	`status` text DEFAULT 'pending' NOT NULL,
	`token` text,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `project_media` (
	`id` text PRIMARY KEY NOT NULL,
	`project_id` text NOT NULL,
	`display_order` integer DEFAULT 0 NOT NULL,
	`media_type` text NOT NULL,
	`r2_base_path` text NOT NULL,
	`variants` text NOT NULL,
	`blurhash` text,
	`metadata` text,
	`created_at` integer NOT NULL,
	FOREIGN KEY (`project_id`) REFERENCES `projects`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE UNIQUE INDEX `idx_project_media_order` ON `project_media` (`project_id`,`display_order`);--> statement-breakpoint
CREATE TABLE `project_relations` (
	`project_id` text NOT NULL,
	`related_project_id` text NOT NULL,
	`position` integer NOT NULL,
	PRIMARY KEY(`project_id`, `related_project_id`),
	FOREIGN KEY (`project_id`) REFERENCES `projects`(`id`) ON UPDATE no action ON DELETE cascade,
	FOREIGN KEY (`related_project_id`) REFERENCES `projects`(`id`) ON UPDATE no action ON DELETE cascade,
	CONSTRAINT "project_relations_no_self" CHECK("project_relations"."project_id" <> "project_relations"."related_project_id")
);
--> statement-breakpoint
CREATE INDEX `idx_project_relations_order` ON `project_relations` (`project_id`,`position`);--> statement-breakpoint
CREATE TABLE `project_tags` (
	`project_id` text NOT NULL,
	`tag_id` text NOT NULL,
	PRIMARY KEY(`project_id`, `tag_id`),
	FOREIGN KEY (`project_id`) REFERENCES `projects`(`id`) ON UPDATE no action ON DELETE cascade,
	FOREIGN KEY (`tag_id`) REFERENCES `tags`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `idx_project_tags_tag` ON `project_tags` (`tag_id`);--> statement-breakpoint
CREATE TABLE `projects` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`short_description` text DEFAULT '' NOT NULL,
	`status` text DEFAULT 'active' NOT NULL,
	`hidden` integer DEFAULT false NOT NULL,
	`private` integer DEFAULT false NOT NULL,
	`github_repo` text,
	`github_repo_id` integer,
	`demo_url` text,
	`project_type` text,
	`accent_color` text,
	`terminal_cast` text,
	`detail_content` text,
	`rendered_html` text,
	`toc` text,
	`last_github_activity` integer,
	`github_synced_at` integer,
	`github_sync_error` text,
	`github_next_sync_at` integer,
	`github_error_count` integer DEFAULT 0 NOT NULL,
	`github_etag` text,
	`github_etag_value` integer,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `projects_slug_unique` ON `projects` (`slug`);--> statement-breakpoint
CREATE INDEX `idx_projects_next_sync` ON `projects` (`github_next_sync_at`);--> statement-breakpoint
CREATE TABLE `sessions` (
	`id` text PRIMARY KEY NOT NULL,
	`user_id` integer NOT NULL,
	`token_hash` text NOT NULL,
	`kind` text NOT NULL,
	`label` text,
	`created_at` integer NOT NULL,
	`expires_at` integer NOT NULL,
	`last_active_at` integer NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE UNIQUE INDEX `sessions_token_hash_unique` ON `sessions` (`token_hash`);--> statement-breakpoint
CREATE INDEX `idx_sessions_user` ON `sessions` (`user_id`);--> statement-breakpoint
CREATE INDEX `idx_sessions_expires` ON `sessions` (`expires_at`);--> statement-breakpoint
CREATE TABLE `site_identity` (
	`id` integer PRIMARY KEY NOT NULL,
	`display_name` text NOT NULL,
	`occupation` text NOT NULL,
	`bio` text NOT NULL,
	`site_title` text NOT NULL,
	`resume_r2_key` text,
	`resume_uploaded_at` integer,
	`updated_at` integer NOT NULL,
	CONSTRAINT "site_identity_single_row" CHECK("site_identity"."id" = 1)
);
--> statement-breakpoint
CREATE TABLE `social_links` (
	`id` text PRIMARY KEY NOT NULL,
	`platform` text NOT NULL,
	`label` text NOT NULL,
	`value` text NOT NULL,
	`icon` text NOT NULL,
	`icon_svg` text,
	`visible` integer DEFAULT true NOT NULL,
	`display_order` integer DEFAULT 0 NOT NULL
);
--> statement-breakpoint
CREATE INDEX `idx_social_links_order` ON `social_links` (`display_order`);--> statement-breakpoint
CREATE TABLE `tags` (
	`id` text PRIMARY KEY NOT NULL,
	`slug` text NOT NULL,
	`name` text NOT NULL,
	`icon` text,
	`icon_svg` text,
	`color` text,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `tags_slug_unique` ON `tags` (`slug`);--> statement-breakpoint
CREATE UNIQUE INDEX `idx_tags_name_lower` ON `tags` (lower("name"));--> statement-breakpoint
CREATE TABLE `users` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`username` text NOT NULL,
	`password_hash` text NOT NULL,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `users_username_unique` ON `users` (`username`);