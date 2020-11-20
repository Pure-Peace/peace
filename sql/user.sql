/*
 Navicat Premium Data Transfer

 Source Server         : pg-localhost
 Source Server Type    : PostgreSQL
 Source Server Version : 110009
 Source Host           : localhost:5432
 Source Catalog        : peace
 Source Schema         : user

 Target Server Type    : PostgreSQL
 Target Server Version : 110009
 File Encoding         : 65001

 Date: 21/11/2020 05:08:45
*/


-- ----------------------------
-- Sequence structure for address_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "user"."address_id_seq";
CREATE SEQUENCE "user"."address_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 2147483647
START 1
CACHE 1;

-- ----------------------------
-- Sequence structure for base_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "user"."base_id_seq";
CREATE SEQUENCE "user"."base_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 2147483647
START 1
CACHE 1;

-- ----------------------------
-- Sequence structure for login_records_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "user"."login_records_id_seq";
CREATE SEQUENCE "user"."login_records_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 2147483647
START 1
CACHE 1;

-- ----------------------------
-- Sequence structure for notes_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "user"."notes_id_seq";
CREATE SEQUENCE "user"."notes_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 2147483647
START 1
CACHE 1;

-- ----------------------------
-- Table structure for address
-- ----------------------------
DROP TABLE IF EXISTS "user"."address";
CREATE TABLE "user"."address" (
  "id" int4 NOT NULL DEFAULT nextval('"user".address_id_seq'::regclass),
  "user_id" int4 NOT NULL,
  "ip" varchar(255) COLLATE "pg_catalog"."default",
  "version" varchar(255) COLLATE "pg_catalog"."default",
  "time_offset" varchar(255) COLLATE "pg_catalog"."default",
  "location" varchar(255) COLLATE "pg_catalog"."default",
  "path" varchar(255) COLLATE "pg_catalog"."default",
  "adapters" text COLLATE "pg_catalog"."default",
  "adapters_hash" varchar(255) COLLATE "pg_catalog"."default",
  "uninstall_id" varchar COLLATE "pg_catalog"."default",
  "disk_id" varchar(255) COLLATE "pg_catalog"."default",
  "create_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
)
;
COMMENT ON COLUMN "user"."address"."id" IS 'address id, unique';
COMMENT ON COLUMN "user"."address"."user_id" IS 'user_id, int 32';
COMMENT ON COLUMN "user"."address"."ip" IS 'ip address';
COMMENT ON COLUMN "user"."address"."version" IS 'osu_version';
COMMENT ON COLUMN "user"."address"."time_offset" IS 'time_offset';
COMMENT ON COLUMN "user"."address"."location" IS 'location';
COMMENT ON COLUMN "user"."address"."path" IS 'osu_path hash';
COMMENT ON COLUMN "user"."address"."adapters" IS 'network physical addresses delimited by ''.''';
COMMENT ON COLUMN "user"."address"."adapters_hash" IS 'adapters_hash';
COMMENT ON COLUMN "user"."address"."uninstall_id" IS 'uniqueid1';
COMMENT ON COLUMN "user"."address"."disk_id" IS 'uniqueid2';
COMMENT ON COLUMN "user"."address"."create_time" IS 'create_time';

-- ----------------------------
-- Records of address
-- ----------------------------

-- ----------------------------
-- Table structure for base
-- ----------------------------
DROP TABLE IF EXISTS "user"."base";
CREATE TABLE "user"."base" (
  "id" int4 NOT NULL DEFAULT nextval('"user".base_id_seq'::regclass),
  "name" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "name_safe" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "password" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "email" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "privileges" int4 NOT NULL DEFAULT 1,
  "country" varchar(255) COLLATE "pg_catalog"."default" NOT NULL DEFAULT 'UN'::character varying,
  "create_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "update_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
)
;
COMMENT ON COLUMN "user"."base"."id" IS 'user_id, int 32, unique';
COMMENT ON COLUMN "user"."base"."name" IS 'username (unsafe), string, unique';
COMMENT ON COLUMN "user"."base"."name_safe" IS 'username (safe), string, unique';
COMMENT ON COLUMN "user"."base"."password" IS 'user password';
COMMENT ON COLUMN "user"."base"."email" IS 'email, string, unique';
COMMENT ON COLUMN "user"."base"."privileges" IS 'user privileges';
COMMENT ON COLUMN "user"."base"."country" IS 'user country';
COMMENT ON COLUMN "user"."base"."create_time" IS 'user create time, auto create';
COMMENT ON COLUMN "user"."base"."update_time" IS 'user info last update time, auto create and update';

-- ----------------------------
-- Records of base
-- ----------------------------

-- ----------------------------
-- Table structure for friends
-- ----------------------------
DROP TABLE IF EXISTS "user"."friends";
CREATE TABLE "user"."friends" (
  "user_id" int4 NOT NULL,
  "friend_id" int4 NOT NULL
)
;
COMMENT ON COLUMN "user"."friends"."user_id" IS 'user_id, int 32';
COMMENT ON COLUMN "user"."friends"."friend_id" IS 'user_id, int 32';

-- ----------------------------
-- Records of friends
-- ----------------------------

-- ----------------------------
-- Table structure for login_records
-- ----------------------------
DROP TABLE IF EXISTS "user"."login_records";
CREATE TABLE "user"."login_records" (
  "id" int4 NOT NULL DEFAULT nextval('"user".login_records_id_seq'::regclass),
  "user_id" int4 NOT NULL,
  "address_id" int4 NOT NULL,
  "count" int4 NOT NULL DEFAULT 1,
  "timestamps" varchar[] COLLATE "pg_catalog"."default" NOT NULL,
  "create_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "update_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
)
;
COMMENT ON COLUMN "user"."login_records"."id" IS 'login_records id, unique';
COMMENT ON COLUMN "user"."login_records"."user_id" IS 'user.id, int 32';
COMMENT ON COLUMN "user"."login_records"."address_id" IS 'user.address.id';
COMMENT ON COLUMN "user"."login_records"."count" IS 'address login count';
COMMENT ON COLUMN "user"."login_records"."timestamps" IS 'each login timestamps';
COMMENT ON COLUMN "user"."login_records"."create_time" IS 'create_time, auto';
COMMENT ON COLUMN "user"."login_records"."update_time" IS 'update_time, auto';

-- ----------------------------
-- Records of login_records
-- ----------------------------

-- ----------------------------
-- Table structure for notes
-- ----------------------------
DROP TABLE IF EXISTS "user"."notes";
CREATE TABLE "user"."notes" (
  "id" int4 NOT NULL DEFAULT nextval('"user".notes_id_seq'::regclass),
  "user_id" int4 NOT NULL,
  "note" text COLLATE "pg_catalog"."default" NOT NULL,
  "type" int4 NOT NULL DEFAULT 0,
  "added_by" int4,
  "create_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "update_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
)
;
COMMENT ON COLUMN "user"."notes"."id" IS 'note id, unique';
COMMENT ON COLUMN "user"."notes"."user_id" IS 'user_id, int 32';
COMMENT ON COLUMN "user"."notes"."note" IS 'note, string';
COMMENT ON COLUMN "user"."notes"."type" IS 'note type, 0: common, 1: reward, 2: warn, 3: punish, 4: multiple accounts, 5: cheats, 6: not important';
COMMENT ON COLUMN "user"."notes"."added_by" IS 'added by who, user_id or null';
COMMENT ON COLUMN "user"."notes"."create_time" IS 'note create time, auto create';
COMMENT ON COLUMN "user"."notes"."update_time" IS 'note last update time, auto create and update';

-- ----------------------------
-- Records of notes
-- ----------------------------

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "user"."address_id_seq"
OWNED BY "user"."address"."id";
SELECT setval('"user"."address_id_seq"', 1, false);

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "user"."base_id_seq"
OWNED BY "user"."base"."id";
SELECT setval('"user"."base_id_seq"', 1000, false);

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "user"."login_records_id_seq"
OWNED BY "user"."login_records"."id";
SELECT setval('"user"."login_records_id_seq"', 1, false);

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "user"."notes_id_seq"
OWNED BY "user"."notes"."id";
SELECT setval('"user"."notes_id_seq"', 1, false);

-- ----------------------------
-- Uniques structure for table address
-- ----------------------------
ALTER TABLE "user"."address" ADD CONSTRAINT "Address.id" UNIQUE ("id");

-- ----------------------------
-- Primary Key structure for table address
-- ----------------------------
ALTER TABLE "user"."address" ADD CONSTRAINT "address_pkey" PRIMARY KEY ("id", "user_id");

-- ----------------------------
-- Triggers structure for table base
-- ----------------------------
CREATE TRIGGER "update_time_auto" BEFORE UPDATE ON "user"."base"
FOR EACH ROW
EXECUTE PROCEDURE "public"."current_timestamp"();
COMMENT ON TRIGGER "update_time_auto" ON "user"."base" IS 'auto update the update_time after update user info';

-- ----------------------------
-- Primary Key structure for table base
-- ----------------------------
ALTER TABLE "user"."base" ADD CONSTRAINT "base_pkey" PRIMARY KEY ("id");

-- ----------------------------
-- Primary Key structure for table friends
-- ----------------------------
ALTER TABLE "user"."friends" ADD CONSTRAINT "friends_pkey" PRIMARY KEY ("user_id", "friend_id");

-- ----------------------------
-- Triggers structure for table login_records
-- ----------------------------
CREATE TRIGGER "update_time_auto" BEFORE UPDATE ON "user"."login_records"
FOR EACH ROW
EXECUTE PROCEDURE "public"."current_timestamp"();
COMMENT ON TRIGGER "update_time_auto" ON "user"."login_records" IS 'auto update the update_time after update user info';

-- ----------------------------
-- Uniques structure for table login_records
-- ----------------------------
ALTER TABLE "user"."login_records" ADD CONSTRAINT "Login_records.id" UNIQUE ("id");
COMMENT ON CONSTRAINT "Login_records.id" ON "user"."login_records" IS 'Login_records.id';

-- ----------------------------
-- Primary Key structure for table login_records
-- ----------------------------
ALTER TABLE "user"."login_records" ADD CONSTRAINT "login_records_pkey" PRIMARY KEY ("id", "user_id", "address_id");

-- ----------------------------
-- Triggers structure for table notes
-- ----------------------------
CREATE TRIGGER "update_time_auto" BEFORE UPDATE ON "user"."notes"
FOR EACH ROW
EXECUTE PROCEDURE "public"."current_timestamp"();
COMMENT ON TRIGGER "update_time_auto" ON "user"."notes" IS 'auto update the update_time after update note info';

-- ----------------------------
-- Uniques structure for table notes
-- ----------------------------
ALTER TABLE "user"."notes" ADD CONSTRAINT "Note.id" UNIQUE ("id");
COMMENT ON CONSTRAINT "Note.id" ON "user"."notes" IS 'note id should be unique';

-- ----------------------------
-- Primary Key structure for table notes
-- ----------------------------
ALTER TABLE "user"."notes" ADD CONSTRAINT "notes_pkey1" PRIMARY KEY ("id", "user_id");

-- ----------------------------
-- Foreign Keys structure for table address
-- ----------------------------
ALTER TABLE "user"."address" ADD CONSTRAINT "User.id" FOREIGN KEY ("user_id") REFERENCES "user"."base" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;

-- ----------------------------
-- Foreign Keys structure for table friends
-- ----------------------------
ALTER TABLE "user"."friends" ADD CONSTRAINT "User.id" FOREIGN KEY ("user_id") REFERENCES "user"."base" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "user"."friends" ADD CONSTRAINT "User.id (friend)" FOREIGN KEY ("friend_id") REFERENCES "user"."base" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
COMMENT ON CONSTRAINT "User.id" ON "user"."friends" IS 'user_id';
COMMENT ON CONSTRAINT "User.id (friend)" ON "user"."friends" IS 'user_id (friend)';

-- ----------------------------
-- Foreign Keys structure for table login_records
-- ----------------------------
ALTER TABLE "user"."login_records" ADD CONSTRAINT "Address.id" FOREIGN KEY ("address_id") REFERENCES "user"."address" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
COMMENT ON CONSTRAINT "Address.id" ON "user"."login_records" IS 'user_id';

-- ----------------------------
-- Foreign Keys structure for table notes
-- ----------------------------
ALTER TABLE "user"."notes" ADD CONSTRAINT "User.id" FOREIGN KEY ("user_id") REFERENCES "user"."base" ("id") ON DELETE CASCADE ON UPDATE NO ACTION;
ALTER TABLE "user"."notes" ADD CONSTRAINT "User.id (added_by)" FOREIGN KEY ("added_by") REFERENCES "user"."base" ("id") ON DELETE NO ACTION ON UPDATE NO ACTION;
