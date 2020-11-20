/*
 Navicat Premium Data Transfer

 Source Server         : pg-localhost
 Source Server Type    : PostgreSQL
 Source Server Version : 110009
 Source Host           : localhost:5432
 Source Catalog        : peace
 Source Schema         : public

 Target Server Type    : PostgreSQL
 Target Server Version : 110009
 File Encoding         : 65001

 Date: 21/11/2020 05:08:50
*/


-- ----------------------------
-- Function structure for current_timestamp
-- ----------------------------
DROP FUNCTION IF EXISTS "public"."current_timestamp"();
CREATE OR REPLACE FUNCTION "public"."current_timestamp"()
  RETURNS "pg_catalog"."trigger" AS $BODY$BEGIN
	new.update_time = CURRENT_TIMESTAMP;
	RETURN new;
END$BODY$
  LANGUAGE plpgsql VOLATILE
  COST 100;
