SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;
CREATE SCHEMA bancho;
COMMENT ON SCHEMA bancho IS 'Bancho configs';
CREATE SCHEMA game_scores;
COMMENT ON SCHEMA game_scores IS 'User''s game scores (including 4 mode, and vanilla, relax, autopilot tables)';
CREATE SCHEMA game_stats;
COMMENT ON SCHEMA game_stats IS 'User''s game stats (such as PP, ACC, PC, TTH, etc.). including std, catch, taiko, mania and vn, ap, rx.';
CREATE SCHEMA "user";
COMMENT ON SCHEMA "user" IS 'user''s info and base data';
CREATE SCHEMA user_records;
COMMENT ON SCHEMA user_records IS 'user''s records, such as login, rename, etc.';
CREATE FUNCTION public.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	NEW.update_time = CURRENT_TIMESTAMP;
	RETURN NEW;
END$$;
CREATE FUNCTION "user".decrease_friend_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "friends_count" = "friends_count" - 1 WHERE "id" = OLD.user_id;
	RETURN OLD;
END$$;
CREATE FUNCTION "user".decrease_note_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "notes_count" = "notes_count" - 1 WHERE "id" = OLD.user_id;
	RETURN OLD;
END$$;
CREATE FUNCTION "user".increase_friend_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "friends_count" = "friends_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;
CREATE FUNCTION "user".increase_note_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "notes_count" = "notes_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;
CREATE FUNCTION "user".insert_related_on_base_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	INSERT INTO "user"."statistic" ("id") VALUES (NEW.id);
	INSERT INTO "game_stats"."std" ("id") VALUES (NEW.id);
	INSERT INTO "game_stats"."catch" ("id") VALUES (NEW.id);
	INSERT INTO "game_stats"."taiko" ("id") VALUES (NEW.id);
	INSERT INTO "game_stats"."mania" ("id") VALUES (NEW.id);
	RETURN NEW;
END$$;
CREATE FUNCTION "user".safe_user_info() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		NEW.name = REPLACE(BTRIM(NEW.name), '_', ' ');
		NEW.email = LOWER(NEW.email);
		NEW.country = UPPER(NEW.country);
		NEW.name_safe = REPLACE(LOWER(NEW.name), ' ', '_');
		
		--only for user base info update
		IF (TG_OP = 'UPDATE') THEN
			--if renamed, insert into rename_records
			IF OLD.name <> NEW.name THEN
				INSERT INTO "user_records"."rename" ("user_id", "new_name", "old_name") VALUES (NEW.id, NEW.name, OLD.name);
			END IF;
		END IF;
	RETURN NEW;
END$$;
CREATE FUNCTION user_records.auto_online_duration() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	IF (NEW.create_time IS NOT NULL) AND (NEW.logout_time IS NOT NULL) THEN
		NEW.online_duration = NEW.logout_time - NEW.create_time;
		UPDATE "user"."statistic" SET "online_duration" = "online_duration" + NEW.online_duration WHERE "id" = NEW.user_id;
	END IF;
	RETURN NEW;
END$$;
CREATE FUNCTION user_records.increase_login_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "login_count" = "login_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;
CREATE FUNCTION user_records.increase_rename_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "rename_count" = "rename_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;
SET default_tablespace = '';
SET default_with_oids = false;
CREATE TABLE bancho.channels (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    title character varying(255) NOT NULL,
    read_priv integer DEFAULT 1 NOT NULL,
    write_priv integer DEFAULT 2 NOT NULL,
    auto_join boolean DEFAULT false NOT NULL,
    create_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN bancho.channels.id IS 'unique channel id';
COMMENT ON COLUMN bancho.channels.name IS 'channel name';
COMMENT ON COLUMN bancho.channels.title IS 'channel title (topic)';
COMMENT ON COLUMN bancho.channels.read_priv IS 'privileges';
COMMENT ON COLUMN bancho.channels.write_priv IS 'privileges';
COMMENT ON COLUMN bancho.channels.auto_join IS 'auto join channel when login';
COMMENT ON COLUMN bancho.channels.create_time IS 'create time';
COMMENT ON COLUMN bancho.channels.update_time IS 'update time';
CREATE SEQUENCE bancho.channels_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE bancho.channels_id_seq OWNED BY bancho.channels.id;
CREATE TABLE game_scores.catch (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.catch.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.catch.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.catch.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.catch.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.catch.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.catch.mods IS 'play mods';
COMMENT ON COLUMN game_scores.catch.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.catch.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.catch.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.catch.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.catch.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.catch.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.catch.update_time IS 'last update time';
CREATE SEQUENCE game_scores.catch_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.catch_id_seq OWNED BY game_scores.catch.id;
CREATE TABLE game_scores.catch_rx (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.catch_rx.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.catch_rx.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.catch_rx.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.catch_rx.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.catch_rx.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.catch_rx.mods IS 'play mods';
COMMENT ON COLUMN game_scores.catch_rx.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.catch_rx.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.catch_rx.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.catch_rx.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.catch_rx.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.catch_rx.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.catch_rx.update_time IS 'last update time';
CREATE SEQUENCE game_scores.catch_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.catch_rx_id_seq OWNED BY game_scores.catch_rx.id;
CREATE TABLE game_scores.mania (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.mania.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.mania.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.mania.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.mania.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.mania.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.mania.mods IS 'play mods';
COMMENT ON COLUMN game_scores.mania.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.mania.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.mania.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.mania.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.mania.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.mania.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.mania.update_time IS 'last update time';
CREATE SEQUENCE game_scores.mania_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.mania_id_seq OWNED BY game_scores.mania.id;
CREATE TABLE game_scores.std (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.std.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.std.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.std.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.std.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.std.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.std.mods IS 'play mods';
COMMENT ON COLUMN game_scores.std.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.std.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.std.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.std.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.std.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.std.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.std.update_time IS 'last update time';
CREATE TABLE game_scores.std_ap (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.std_ap.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.std_ap.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.std_ap.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.std_ap.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.std_ap.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.std_ap.mods IS 'play mods';
COMMENT ON COLUMN game_scores.std_ap.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.std_ap.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.std_ap.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.std_ap.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.std_ap.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.std_ap.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.std_ap.update_time IS 'last update time';
CREATE SEQUENCE game_scores.std_ap_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.std_ap_id_seq OWNED BY game_scores.std_ap.id;
CREATE SEQUENCE game_scores.std_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.std_id_seq OWNED BY game_scores.std.id;
CREATE TABLE game_scores.std_rx (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.std_rx.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.std_rx.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.std_rx.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.std_rx.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.std_rx.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.std_rx.mods IS 'play mods';
COMMENT ON COLUMN game_scores.std_rx.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.std_rx.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.std_rx.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.std_rx.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.std_rx.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.std_rx.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.std_rx.update_time IS 'last update time';
CREATE SEQUENCE game_scores.std_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.std_rx_id_seq OWNED BY game_scores.std_rx.id;
CREATE TABLE game_scores.taiko (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.taiko.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.taiko.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.taiko.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.taiko.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.taiko.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.taiko.mods IS 'play mods';
COMMENT ON COLUMN game_scores.taiko.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.taiko.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.taiko.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.taiko.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.taiko.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.taiko.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.taiko.update_time IS 'last update time';
CREATE SEQUENCE game_scores.taiko_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.taiko_id_seq OWNED BY game_scores.taiko.id;
CREATE TABLE game_scores.taiko_rx (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    map_md5 character varying(32) NOT NULL,
    score integer NOT NULL,
    performance_v1 real NOT NULL,
    performance_v2 real NOT NULL,
    accuracy real NOT NULL,
    combo integer NOT NULL,
    mods integer NOT NULL,
    n300 integer NOT NULL,
    n100 integer NOT NULL,
    n50 integer NOT NULL,
    miss integer NOT NULL,
    geiki integer NOT NULL,
    katu integer NOT NULL,
    playtime integer NOT NULL,
    perfect boolean DEFAULT false NOT NULL,
    client_version character varying(255) NOT NULL,
    confidence smallint DEFAULT 100 NOT NULL,
    verified boolean DEFAULT false NOT NULL,
    checked boolean DEFAULT false NOT NULL,
    check_time timestamp(6) with time zone,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN game_scores.taiko_rx.id IS 'score''s unique id';
COMMENT ON COLUMN game_scores.taiko_rx.user_id IS 'user''s unique id';
COMMENT ON COLUMN game_scores.taiko_rx.map_md5 IS 'beatmap''s md5';
COMMENT ON COLUMN game_scores.taiko_rx.performance_v1 IS 'ppv1';
COMMENT ON COLUMN game_scores.taiko_rx.performance_v2 IS 'ppv2';
COMMENT ON COLUMN game_scores.taiko_rx.mods IS 'play mods';
COMMENT ON COLUMN game_scores.taiko_rx.playtime IS 'play time (seconds)';
COMMENT ON COLUMN game_scores.taiko_rx.perfect IS 'this score is full combo or not';
COMMENT ON COLUMN game_scores.taiko_rx.client_version IS 'the client version used to submit this score';
COMMENT ON COLUMN game_scores.taiko_rx.confidence IS 'credibility of score';
COMMENT ON COLUMN game_scores.taiko_rx.check_time IS 'last check time';
COMMENT ON COLUMN game_scores.taiko_rx.create_time IS 'submission time';
COMMENT ON COLUMN game_scores.taiko_rx.update_time IS 'last update time';
CREATE SEQUENCE game_scores.taiko_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE game_scores.taiko_rx_id_seq OWNED BY game_scores.taiko_rx.id;
CREATE TABLE game_stats.catch (
    id integer NOT NULL,
    total_score bigint DEFAULT 0 NOT NULL,
    ranked_score bigint DEFAULT 0 NOT NULL,
    total_score_rx bigint DEFAULT 0 NOT NULL,
    ranked_score_rx bigint DEFAULT 0 NOT NULL,
    performance_v1 smallint DEFAULT 0 NOT NULL,
    performance_v2 smallint DEFAULT 0 NOT NULL,
    performance_v1_rx smallint DEFAULT 0 NOT NULL,
    performance_v2_rx smallint DEFAULT 0 NOT NULL,
    playcount integer DEFAULT 0 NOT NULL,
    playcount_rx integer DEFAULT 0 NOT NULL,
    total_hits integer DEFAULT 0 NOT NULL,
    total_hits_rx integer DEFAULT 0 NOT NULL,
    accuracy real DEFAULT 0.0 NOT NULL,
    accuracy_rx real DEFAULT 0.0 NOT NULL,
    maxcombo integer DEFAULT 0 NOT NULL,
    maxcombo_rx integer DEFAULT 0 NOT NULL,
    playtime bigint DEFAULT 0 NOT NULL,
    playtime_rx bigint DEFAULT 0 NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE game_stats.catch IS 'Catch the beat (including vanilla, relax)';
COMMENT ON COLUMN game_stats.catch.id IS 'user''s unique id';
CREATE TABLE game_stats.mania (
    id integer NOT NULL,
    total_score bigint DEFAULT 0 NOT NULL,
    ranked_score bigint DEFAULT 0 NOT NULL,
    performance_v1 smallint DEFAULT 0 NOT NULL,
    performance_v2 smallint DEFAULT 0 NOT NULL,
    playcount integer DEFAULT 0 NOT NULL,
    total_hits integer DEFAULT 0 NOT NULL,
    accuracy real DEFAULT 0.0 NOT NULL,
    maxcombo integer DEFAULT 0 NOT NULL,
    playtime bigint DEFAULT 0 NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE game_stats.mania IS 'Mania (only vanilla)';
COMMENT ON COLUMN game_stats.mania.id IS 'user''s unique id';
CREATE TABLE game_stats.std (
    id integer NOT NULL,
    total_score bigint DEFAULT 0 NOT NULL,
    ranked_score bigint DEFAULT 0 NOT NULL,
    total_score_rx bigint DEFAULT 0 NOT NULL,
    ranked_score_rx bigint DEFAULT 0 NOT NULL,
    total_score_ap bigint DEFAULT 0 NOT NULL,
    ranked_score_ap bigint DEFAULT 0 NOT NULL,
    performance_v1 smallint DEFAULT 0 NOT NULL,
    performance_v2 smallint DEFAULT 0 NOT NULL,
    performance_v1_rx smallint DEFAULT 0 NOT NULL,
    performance_v2_rx smallint DEFAULT 0 NOT NULL,
    performance_v1_ap smallint DEFAULT 0 NOT NULL,
    performance_v2_ap smallint DEFAULT 0 NOT NULL,
    playcount integer DEFAULT 0 NOT NULL,
    playcount_rx integer DEFAULT 0 NOT NULL,
    playcount_ap integer DEFAULT 0 NOT NULL,
    total_hits integer DEFAULT 0 NOT NULL,
    total_hits_rx integer DEFAULT 0 NOT NULL,
    total_hits_ap integer DEFAULT 0 NOT NULL,
    accuracy real DEFAULT 0.0 NOT NULL,
    accuracy_rx real DEFAULT 0.0 NOT NULL,
    accuracy_ap real DEFAULT 0.0 NOT NULL,
    maxcombo integer DEFAULT 0 NOT NULL,
    maxcombo_rx integer DEFAULT 0 NOT NULL,
    maxcombo_ap integer DEFAULT 0 NOT NULL,
    playtime bigint DEFAULT 0 NOT NULL,
    playtime_rx bigint DEFAULT 0 NOT NULL,
    playtime_ap bigint DEFAULT 0 NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE game_stats.std IS 'Standard (including vanilla, relax, autopilot)';
COMMENT ON COLUMN game_stats.std.id IS 'user''s unique id';
CREATE TABLE game_stats.taiko (
    id integer NOT NULL,
    total_score bigint DEFAULT 0 NOT NULL,
    ranked_score bigint DEFAULT 0 NOT NULL,
    total_score_rx bigint DEFAULT 0 NOT NULL,
    ranked_score_rx bigint DEFAULT 0 NOT NULL,
    performance_v1 smallint DEFAULT 0 NOT NULL,
    performance_v2 smallint DEFAULT 0 NOT NULL,
    performance_v1_rx smallint DEFAULT 0 NOT NULL,
    performance_v2_rx smallint DEFAULT 0 NOT NULL,
    playcount integer DEFAULT 0 NOT NULL,
    playcount_rx integer DEFAULT 0 NOT NULL,
    total_hits integer DEFAULT 0 NOT NULL,
    total_hits_rx integer DEFAULT 0 NOT NULL,
    accuracy real DEFAULT 0.0 NOT NULL,
    accuracy_rx real DEFAULT 0.0 NOT NULL,
    maxcombo integer DEFAULT 0 NOT NULL,
    maxcombo_rx integer DEFAULT 0 NOT NULL,
    playtime bigint DEFAULT 0 NOT NULL,
    playtime_rx bigint DEFAULT 0 NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE game_stats.taiko IS 'Taiko (including vanilla, relax)';
COMMENT ON COLUMN game_stats.taiko.id IS 'user''s unique id';
CREATE TABLE public.db_versions (
    version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    author character varying(255) DEFAULT 'PurePeace'::character varying NOT NULL,
    sql text,
    release_note text,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN public.db_versions.version IS 'peace database version';
COMMENT ON COLUMN public.db_versions.author IS 'version publisher';
COMMENT ON COLUMN public.db_versions.sql IS 'database initial sql';
COMMENT ON COLUMN public.db_versions.release_note IS 'version release note';
CREATE TABLE public.versions (
    version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    author character varying(255) DEFAULT 'PurePeace'::character varying NOT NULL,
    db_version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    release_note text,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON COLUMN public.versions.version IS 'peace version';
COMMENT ON COLUMN public.versions.author IS 'version publisher';
COMMENT ON COLUMN public.versions.db_version IS 'peace ''s database version';
COMMENT ON COLUMN public.versions.release_note IS 'version release note';
CREATE TABLE "user".address (
    id integer NOT NULL,
    user_id integer NOT NULL,
    time_offset integer NOT NULL,
    path character varying(255) NOT NULL,
    adapters text NOT NULL,
    adapters_hash character varying(255) NOT NULL,
    uninstall_id character varying(255) NOT NULL,
    disk_id character varying(255) NOT NULL,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE "user".address IS 'User''s login hardware address';
COMMENT ON COLUMN "user".address.id IS 'address id, unique';
COMMENT ON COLUMN "user".address.user_id IS 'user_id, int 32';
COMMENT ON COLUMN "user".address.time_offset IS 'time_offset';
COMMENT ON COLUMN "user".address.path IS 'osu_path hash';
COMMENT ON COLUMN "user".address.adapters IS 'network physical addresses delimited by ''.''';
COMMENT ON COLUMN "user".address.adapters_hash IS 'adapters_hash';
COMMENT ON COLUMN "user".address.uninstall_id IS 'uniqueid1';
COMMENT ON COLUMN "user".address.disk_id IS 'uniqueid2';
COMMENT ON COLUMN "user".address.create_time IS 'create_time';
CREATE SEQUENCE "user".address_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE "user".address_id_seq OWNED BY "user".address.id;
CREATE TABLE "user".base (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    name_safe character varying(255) NOT NULL,
    password character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    privileges integer DEFAULT 1 NOT NULL,
    country character varying(255) DEFAULT 'UN'::character varying NOT NULL,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE "user".base IS 'Basic user information, such as user name, password, email, etc.';
COMMENT ON COLUMN "user".base.id IS 'user_id, int 32, unique';
COMMENT ON COLUMN "user".base.name IS 'username (unsafe), string, unique';
COMMENT ON COLUMN "user".base.name_safe IS 'username (safe), string, unique';
COMMENT ON COLUMN "user".base.password IS 'user’s Argon2 crypted password hash';
COMMENT ON COLUMN "user".base.email IS 'email, string, unique';
COMMENT ON COLUMN "user".base.privileges IS 'user privileges';
COMMENT ON COLUMN "user".base.country IS 'user country';
COMMENT ON COLUMN "user".base.create_time IS 'user create time, auto create';
COMMENT ON COLUMN "user".base.update_time IS 'user info last update time, auto create and update';
CREATE SEQUENCE "user".base_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;
ALTER SEQUENCE "user".base_id_seq OWNED BY "user".base.id;
CREATE TABLE "user".friends (
    user_id integer NOT NULL,
    friend_id integer NOT NULL,
    remark character varying(255),
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE "user".friends IS 'User’s friends';
COMMENT ON COLUMN "user".friends.user_id IS 'user_id, int 32';
COMMENT ON COLUMN "user".friends.friend_id IS 'user_id, int 32';
COMMENT ON COLUMN "user".friends.remark IS 'friend remark, such as aka';
COMMENT ON COLUMN "user".friends.create_time IS 'create timestamp, auto';
CREATE TABLE "user".notes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    note text NOT NULL,
    type integer DEFAULT 0 NOT NULL,
    added_by integer,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE "user".notes IS 'User’s notes, which may be rewards or warnings etc.';
COMMENT ON COLUMN "user".notes.id IS 'note id, unique';
COMMENT ON COLUMN "user".notes.user_id IS 'user_id, int 32';
COMMENT ON COLUMN "user".notes.note IS 'note, string';
COMMENT ON COLUMN "user".notes.type IS 'note type, 0: common, 1: reward, 2: warn, 3: punish, 4: multiple accounts, 5: cheats, 6: not important';
COMMENT ON COLUMN "user".notes.added_by IS 'added by who, user_id or null';
COMMENT ON COLUMN "user".notes.create_time IS 'note create time, auto create';
COMMENT ON COLUMN "user".notes.update_time IS 'note last update time, auto create and update';
CREATE SEQUENCE "user".notes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;
ALTER SEQUENCE "user".notes_id_seq OWNED BY "user".notes.id;
CREATE TABLE "user".statistic (
    id integer NOT NULL,
    online_duration interval DEFAULT '00:00:00'::interval NOT NULL,
    login_count integer DEFAULT 0 NOT NULL,
    rename_count integer DEFAULT 0 NOT NULL,
    friends_count integer DEFAULT 0 NOT NULL,
    notes_count integer DEFAULT 0 NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE "user".statistic IS 'User''s info statistic (such as online duration, etc.)';
COMMENT ON COLUMN "user".statistic.id IS 'user''s unique id';
COMMENT ON COLUMN "user".statistic.online_duration IS 'user''s total online duration';
COMMENT ON COLUMN "user".statistic.login_count IS 'user''s total login count';
COMMENT ON COLUMN "user".statistic.rename_count IS 'user''s total rename count';
COMMENT ON COLUMN "user".statistic.friends_count IS 'user''s total friend count';
COMMENT ON COLUMN "user".statistic.notes_count IS 'user''s total note count';
COMMENT ON COLUMN "user".statistic.update_time IS 'update time';
CREATE TABLE user_records.login (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    address_id integer NOT NULL,
    ip character varying(255) NOT NULL,
    version character varying(255) NOT NULL,
    similarity integer DEFAULT 101 NOT NULL,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    logout_time timestamp with time zone,
    online_duration interval
);
COMMENT ON TABLE user_records.login IS 'The user''s login record, associated with the user''s login address';
COMMENT ON COLUMN user_records.login.id IS 'login record id';
COMMENT ON COLUMN user_records.login.user_id IS 'user.id, int 32';
COMMENT ON COLUMN user_records.login.address_id IS 'user.address.id';
COMMENT ON COLUMN user_records.login.ip IS 'ip address';
COMMENT ON COLUMN user_records.login.version IS 'osu version';
COMMENT ON COLUMN user_records.login.similarity IS 'certainty of the address';
COMMENT ON COLUMN user_records.login.create_time IS 'create_time, auto';
COMMENT ON COLUMN user_records.login.logout_time IS 'this record''s logout time';
COMMENT ON COLUMN user_records.login.online_duration IS 'online duration';
CREATE SEQUENCE user_records.login_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE user_records.login_records_id_seq OWNED BY user_records.login.id;
CREATE TABLE user_records.rename (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    new_name character varying(255) NOT NULL,
    old_name character varying(255) NOT NULL,
    create_time timestamp(0) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
COMMENT ON TABLE user_records.rename IS 'Automatically record the user''s rename record (do not add manually)';
COMMENT ON COLUMN user_records.rename.id IS 'rename records id';
COMMENT ON COLUMN user_records.rename.user_id IS 'user''s unique id';
COMMENT ON COLUMN user_records.rename.new_name IS 'user''s new name (after rename)';
COMMENT ON COLUMN user_records.rename.old_name IS 'user''s old name (before rename)';
COMMENT ON COLUMN user_records.rename.create_time IS 'rename records create time';
CREATE SEQUENCE user_records.rename_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;
ALTER SEQUENCE user_records.rename_records_id_seq OWNED BY user_records.rename.id;
ALTER TABLE ONLY bancho.channels ALTER COLUMN id SET DEFAULT nextval('bancho.channels_id_seq'::regclass);
ALTER TABLE ONLY game_scores.catch ALTER COLUMN id SET DEFAULT nextval('game_scores.catch_id_seq'::regclass);
ALTER TABLE ONLY game_scores.catch_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.catch_rx_id_seq'::regclass);
ALTER TABLE ONLY game_scores.mania ALTER COLUMN id SET DEFAULT nextval('game_scores.mania_id_seq'::regclass);
ALTER TABLE ONLY game_scores.std ALTER COLUMN id SET DEFAULT nextval('game_scores.std_id_seq'::regclass);
ALTER TABLE ONLY game_scores.std_ap ALTER COLUMN id SET DEFAULT nextval('game_scores.std_ap_id_seq'::regclass);
ALTER TABLE ONLY game_scores.std_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.std_rx_id_seq'::regclass);
ALTER TABLE ONLY game_scores.taiko ALTER COLUMN id SET DEFAULT nextval('game_scores.taiko_id_seq'::regclass);
ALTER TABLE ONLY game_scores.taiko_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.taiko_rx_id_seq'::regclass);
ALTER TABLE ONLY "user".address ALTER COLUMN id SET DEFAULT nextval('"user".address_id_seq'::regclass);
ALTER TABLE ONLY "user".base ALTER COLUMN id SET DEFAULT nextval('"user".base_id_seq'::regclass);
ALTER TABLE ONLY "user".notes ALTER COLUMN id SET DEFAULT nextval('"user".notes_id_seq'::regclass);
ALTER TABLE ONLY user_records.login ALTER COLUMN id SET DEFAULT nextval('user_records.login_records_id_seq'::regclass);
ALTER TABLE ONLY user_records.rename ALTER COLUMN id SET DEFAULT nextval('user_records.rename_records_id_seq'::regclass);
INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (1, '#osu', 'General discussion.', 1, 2, true, '2020-12-09 04:21:05.471552+08', '2020-12-09 04:21:14.652774+08');
INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (3, '#announce', 'Exemplary performance and public announcements.', 1, 2, true, '2020-12-09 04:21:35.551317+08', '2020-12-09 04:21:35.551317+08');
INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (4, '#lobby', 'Multiplayer lobby discussion room.', 1, 2, true, '2020-12-09 04:21:46.339821+08', '2020-12-09 04:21:46.339821+08');
INSERT INTO game_stats.catch (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.catch (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO game_stats.mania (id, total_score, ranked_score, performance_v1, performance_v2, playcount, total_hits, accuracy, maxcombo, playtime, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.mania (id, total_score, ranked_score, performance_v1, performance_v2, playcount, total_hits, accuracy, maxcombo, playtime, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO game_stats.std (id, total_score, ranked_score, total_score_rx, ranked_score_rx, total_score_ap, ranked_score_ap, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, performance_v1_ap, performance_v2_ap, playcount, playcount_rx, playcount_ap, total_hits, total_hits_rx, total_hits_ap, accuracy, accuracy_rx, accuracy_ap, maxcombo, maxcombo_rx, maxcombo_ap, playtime, playtime_rx, playtime_ap, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.std (id, total_score, ranked_score, total_score_rx, ranked_score_rx, total_score_ap, ranked_score_ap, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, performance_v1_ap, performance_v2_ap, playcount, playcount_rx, playcount_ap, total_hits, total_hits_rx, total_hits_ap, accuracy, accuracy_rx, accuracy_ap, maxcombo, maxcombo_rx, maxcombo_ap, playtime, playtime_rx, playtime_ap, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO game_stats.taiko (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.taiko (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO public.db_versions (version, author, sql, release_note, create_time, update_time) VALUES ('0.1.0', 'PurePeace', NULL, 'initial', '2020-12-15 01:15:37.586205+08', '2020-12-20 01:13:47.84393+08');
INSERT INTO public.db_versions (version, author, sql, release_note, create_time, update_time) VALUES ('0.1.3', 'PurePeace', NULL, 'add game_scores, game_stats, modify some column', '2020-12-15 01:15:37.586205+08', '2020-12-15 01:15:52.635208+08');
INSERT INTO public.versions (version, author, db_version, release_note, create_time, update_time) VALUES ('0.1.0', 'PurePeace', '0.1.0', 'initial (wip)', '2020-12-15 01:16:37.785543+08', '2020-12-20 01:16:34.355013+08');
INSERT INTO public.versions (version, author, db_version, release_note, create_time, update_time) VALUES ('0.1.2', 'PurePeace', '0.1.3', 'add tables', '2020-12-15 01:16:37.785543+08', '2020-12-20 01:16:34.355013+08');
INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (2, 'ChinoChan', 'chinochan', '$argon2i$v=19$m=4096,t=3,p=1$bmVQNTdoZmdJSW9nMERsYWd4OGxRZ1hRSFpvUjg5TEs$H6OEckDS9yVSODESGYA2mPudB2UkoBUH8UhVB6B6Dsg', 'a@chino.com', 1, 'JP', '2020-12-19 21:35:54.465545+08', '2020-12-20 01:12:42.56606+08');
INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (1, 'PurePeace', 'purepeace', '$argon2i$v=19$m=4096,t=3,p=1$VGQ3NXNFbnV1a25hVHAzazZwRm80N3hROVFabHdmaHk$djMKitAp+E/PD56gyVnIeM/7HmJNM9xBt6h/yAuRqPk', '940857703@qq.com', 3, 'CN', '2020-12-19 21:35:32.810099+08', '2020-12-20 01:18:58.947387+08');
INSERT INTO "user".statistic (id, online_duration, login_count, rename_count, friends_count, notes_count, update_time) VALUES (2, '00:00:00', 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO "user".statistic (id, online_duration, login_count, rename_count, friends_count, notes_count, update_time) VALUES (1, '00:00:00', 0, 0, 0, 0, '2020-12-20 01:23:57.20465+08');
SELECT pg_catalog.setval('bancho.channels_id_seq', 4, true);
SELECT pg_catalog.setval('game_scores.catch_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.catch_rx_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.mania_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.std_ap_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.std_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.std_rx_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.taiko_id_seq', 1, true);
SELECT pg_catalog.setval('game_scores.taiko_rx_id_seq', 1, true);
SELECT pg_catalog.setval('"user".address_id_seq', 1, true);
SELECT pg_catalog.setval('"user".base_id_seq', 100, true);
SELECT pg_catalog.setval('"user".notes_id_seq', 1, true);
SELECT pg_catalog.setval('user_records.login_records_id_seq', 1, true);
SELECT pg_catalog.setval('user_records.rename_records_id_seq', 1, true);
ALTER TABLE ONLY bancho.channels
    ADD CONSTRAINT "channel.name" UNIQUE (name);
COMMENT ON CONSTRAINT "channel.name" ON bancho.channels IS 'channel name should be unique';
ALTER TABLE ONLY bancho.channels
    ADD CONSTRAINT channels_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.catch_rx
    ADD CONSTRAINT catch_rx_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.catch
    ADD CONSTRAINT catch_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.mania
    ADD CONSTRAINT mania_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.std_ap
    ADD CONSTRAINT std_ap_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.std_rx
    ADD CONSTRAINT std_rx_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.std
    ADD CONSTRAINT std_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.taiko_rx
    ADD CONSTRAINT taiko_rx_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_scores.taiko
    ADD CONSTRAINT taiko_scores_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_stats.catch
    ADD CONSTRAINT catch_stats_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_stats.mania
    ADD CONSTRAINT mania_stats_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_stats.std
    ADD CONSTRAINT std_stats_pkey PRIMARY KEY (id);
ALTER TABLE ONLY game_stats.taiko
    ADD CONSTRAINT taiko_stats_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.db_versions
    ADD CONSTRAINT db_versions_pkey PRIMARY KEY (version);
ALTER TABLE ONLY public.versions
    ADD CONSTRAINT versions_pkey PRIMARY KEY (version);
ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "Note.id" UNIQUE (id);
COMMENT ON CONSTRAINT "Note.id" ON "user".notes IS 'note id should be unique';
ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - email" UNIQUE (email);
COMMENT ON CONSTRAINT "Unique - email" ON "user".base IS 'email should be unique';
ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - name" UNIQUE (name);
ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - name safe" UNIQUE (name_safe);
COMMENT ON CONSTRAINT "Unique - name safe" ON "user".base IS 'name safe should be unique';
ALTER TABLE ONLY "user".address
    ADD CONSTRAINT address_pkey PRIMARY KEY (id);
ALTER TABLE ONLY "user".base
    ADD CONSTRAINT base_pkey PRIMARY KEY (id);
ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT friends_pkey PRIMARY KEY (user_id, friend_id);
ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT notes_pkey PRIMARY KEY (id, user_id);
ALTER TABLE ONLY "user".statistic
    ADD CONSTRAINT statistic_pkey PRIMARY KEY (id);
ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT login_records_pkey PRIMARY KEY (id);
ALTER TABLE ONLY user_records.rename
    ADD CONSTRAINT rename_records_pkey PRIMARY KEY (id);
CREATE UNIQUE INDEX "User.name" ON "user".base USING btree (name, name_safe);
CREATE INDEX user_address ON "user".address USING btree (user_id);
CREATE TRIGGER auto_update_time BEFORE UPDATE ON bancho.channels FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.catch FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.catch IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.catch_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.catch_rx IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.mania FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.mania IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.std IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std_ap FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.std_ap IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.std_rx IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.taiko FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.taiko IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.taiko_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_scores.taiko_rx IS 'auto update time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.catch FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_stats.catch IS 'auto update the time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.mania FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_stats.mania IS 'auto update the time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.std FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_stats.std IS 'auto update the time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.taiko FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON game_stats.taiko IS 'auto update the time';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON public.db_versions FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
CREATE TRIGGER auto_update_time BEFORE UPDATE ON public.versions FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
CREATE TRIGGER auto_insert_related AFTER INSERT ON "user".base FOR EACH ROW EXECUTE PROCEDURE "user".insert_related_on_base_insert();
COMMENT ON TRIGGER auto_insert_related ON "user".base IS 'auto insert into related table';
CREATE TRIGGER auto_update_time BEFORE UPDATE ON "user".statistic FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_time ON "user".statistic IS 'auto update the timestamp';
CREATE TRIGGER auto_update_timestamp BEFORE UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER auto_update_timestamp ON "user".base IS 'auto update the update_time after update user info';
CREATE TRIGGER decrease_friend_count AFTER DELETE ON "user".friends FOR EACH ROW EXECUTE PROCEDURE "user".decrease_friend_count();
COMMENT ON TRIGGER decrease_friend_count ON "user".friends IS 'update the statistic';
CREATE TRIGGER decrease_note_count AFTER DELETE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE "user".decrease_note_count();
COMMENT ON TRIGGER decrease_note_count ON "user".notes IS 'update the statistic';
CREATE TRIGGER increase_friend_count AFTER INSERT ON "user".friends FOR EACH ROW EXECUTE PROCEDURE "user".increase_friend_count();
COMMENT ON TRIGGER increase_friend_count ON "user".friends IS 'update the statistic';
CREATE TRIGGER increase_note_count AFTER INSERT ON "user".notes FOR EACH ROW EXECUTE PROCEDURE "user".increase_note_count();
COMMENT ON TRIGGER increase_note_count ON "user".notes IS 'update the statistic';
CREATE TRIGGER safe_user_info BEFORE INSERT OR UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE "user".safe_user_info();
COMMENT ON TRIGGER safe_user_info ON "user".base IS 'auto make the user info safety';
CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();
COMMENT ON TRIGGER update_time_auto ON "user".notes IS 'auto update the update_time after update note info';
CREATE TRIGGER auto_login_duration BEFORE UPDATE ON user_records.login FOR EACH ROW EXECUTE PROCEDURE user_records.auto_online_duration();
COMMENT ON TRIGGER auto_login_duration ON user_records.login IS 'auto update the online duration';
CREATE TRIGGER increase_login_count BEFORE INSERT ON user_records.login FOR EACH ROW EXECUTE PROCEDURE user_records.increase_login_count();
COMMENT ON TRIGGER increase_login_count ON user_records.login IS 'auto update the statistic';
CREATE TRIGGER increase_rename_count BEFORE INSERT ON user_records.rename FOR EACH ROW EXECUTE PROCEDURE user_records.increase_rename_count();
COMMENT ON TRIGGER increase_rename_count ON user_records.rename IS 'update user statistic';
ALTER TABLE ONLY game_stats.catch
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY game_stats.mania
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY game_stats.std
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY game_stats.taiko
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.versions
    ADD CONSTRAINT db_version FOREIGN KEY (db_version) REFERENCES public.db_versions(version) ON UPDATE CASCADE ON DELETE RESTRICT;
ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
COMMENT ON CONSTRAINT "User.id" ON "user".friends IS 'user_id';
ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY "user".address
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY "user".statistic
    ADD CONSTRAINT "User.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
COMMENT ON CONSTRAINT "User.id" ON "user".statistic IS 'user''s unique id';
ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id (added_by)" FOREIGN KEY (added_by) REFERENCES "user".base(id) ON UPDATE CASCADE;
ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id (friend)" FOREIGN KEY (friend_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
COMMENT ON CONSTRAINT "User.id (friend)" ON "user".friends IS 'user_id (friend)';
ALTER TABLE ONLY user_records.rename
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
COMMENT ON CONSTRAINT "User.id" ON user_records.rename IS 'user''s unique id';
ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT "address.id" FOREIGN KEY (address_id) REFERENCES "user".address(id) ON UPDATE CASCADE ON DELETE CASCADE;
