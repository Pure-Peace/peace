--
-- PostgreSQL database dump
--

-- Dumped from database version 11.9
-- Dumped by pg_dump version 11.9

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

--
-- Name: bancho; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA bancho;


--
-- Name: game_scores; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA game_scores;


--
-- Name: game_stats; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA game_stats;


--
-- Name: user; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA "user";


--
-- Name: SCHEMA "user"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA "user" IS 'user''s info';


--
-- Name: user_records; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA user_records;


--
-- Name: SCHEMA user_records; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA user_records IS 'user''s records';


--
-- Name: update_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	NEW.update_time = CURRENT_TIMESTAMP;
	RETURN NEW;
END$$;


--
-- Name: decrease_friend_count(); Type: FUNCTION; Schema: user; Owner: -
--

CREATE FUNCTION "user".decrease_friend_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "friends_count" = "friends_count" - 1 WHERE "id" = OLD.user_id;
	RETURN OLD;
END$$;


--
-- Name: decrease_note_count(); Type: FUNCTION; Schema: user; Owner: -
--

CREATE FUNCTION "user".decrease_note_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "notes_count" = "notes_count" - 1 WHERE "id" = OLD.user_id;
	RETURN OLD;
END$$;


--
-- Name: increase_friend_count(); Type: FUNCTION; Schema: user; Owner: -
--

CREATE FUNCTION "user".increase_friend_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "friends_count" = "friends_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;


--
-- Name: increase_note_count(); Type: FUNCTION; Schema: user; Owner: -
--

CREATE FUNCTION "user".increase_note_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "notes_count" = "notes_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;


--
-- Name: insert_related_on_base_insert(); Type: FUNCTION; Schema: user; Owner: -
--

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


--
-- Name: safe_user_info(); Type: FUNCTION; Schema: user; Owner: -
--

CREATE FUNCTION "user".safe_user_info() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		NEW.name = REPLACE(BTRIM(NEW.name), '_', ' ');
		NEW.email = LOWER(NEW.email);
		NEW.country = UPPER(NEW.country);
		NEW.password = LOWER(NEW.password);
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


--
-- Name: auto_online_duration(); Type: FUNCTION; Schema: user_records; Owner: -
--

CREATE FUNCTION user_records.auto_online_duration() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	IF (NEW.create_time IS NOT NULL) AND (NEW.logout_time IS NOT NULL) THEN
		NEW.online_duration = NEW.logout_time - NEW.create_time;
		UPDATE "user"."statistic" SET "online_duration" = "online_duration" + NEW.online_duration WHERE "id" = NEW.user_id;
	END IF;
	RETURN NEW;
END$$;


--
-- Name: increase_login_count(); Type: FUNCTION; Schema: user_records; Owner: -
--

CREATE FUNCTION user_records.increase_login_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "login_count" = "login_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;


--
-- Name: increase_rename_count(); Type: FUNCTION; Schema: user_records; Owner: -
--

CREATE FUNCTION user_records.increase_rename_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
		UPDATE "user"."statistic" SET "rename_count" = "rename_count" + 1 WHERE "id" = NEW.user_id;
	RETURN NEW;
END$$;


SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: channels; Type: TABLE; Schema: bancho; Owner: -
--

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


--
-- Name: COLUMN channels.id; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.id IS 'unique channel id';


--
-- Name: COLUMN channels.name; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.name IS 'channel name';


--
-- Name: COLUMN channels.title; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.title IS 'channel title (topic)';


--
-- Name: COLUMN channels.read_priv; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.read_priv IS 'privileges';


--
-- Name: COLUMN channels.write_priv; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.write_priv IS 'privileges';


--
-- Name: COLUMN channels.auto_join; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.auto_join IS 'auto join channel when login';


--
-- Name: COLUMN channels.create_time; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.create_time IS 'create time';


--
-- Name: COLUMN channels.update_time; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON COLUMN bancho.channels.update_time IS 'update time';


--
-- Name: channels_id_seq; Type: SEQUENCE; Schema: bancho; Owner: -
--

CREATE SEQUENCE bancho.channels_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: channels_id_seq; Type: SEQUENCE OWNED BY; Schema: bancho; Owner: -
--

ALTER SEQUENCE bancho.channels_id_seq OWNED BY bancho.channels.id;


--
-- Name: catch; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN catch.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.id IS 'score''s unique id';


--
-- Name: COLUMN catch.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.user_id IS 'user''s unique id';


--
-- Name: COLUMN catch.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN catch.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.performance_v1 IS 'ppv1';


--
-- Name: COLUMN catch.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.performance_v2 IS 'ppv2';


--
-- Name: COLUMN catch.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.mods IS 'play mods';


--
-- Name: COLUMN catch.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.playtime IS 'play time (seconds)';


--
-- Name: COLUMN catch.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN catch.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN catch.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.confidence IS 'credibility of score';


--
-- Name: COLUMN catch.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.check_time IS 'last check time';


--
-- Name: COLUMN catch.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.create_time IS 'submission time';


--
-- Name: COLUMN catch.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch.update_time IS 'last update time';


--
-- Name: catch_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.catch_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: catch_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.catch_id_seq OWNED BY game_scores.catch.id;


--
-- Name: catch_rx; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN catch_rx.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.id IS 'score''s unique id';


--
-- Name: COLUMN catch_rx.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.user_id IS 'user''s unique id';


--
-- Name: COLUMN catch_rx.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN catch_rx.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.performance_v1 IS 'ppv1';


--
-- Name: COLUMN catch_rx.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.performance_v2 IS 'ppv2';


--
-- Name: COLUMN catch_rx.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.mods IS 'play mods';


--
-- Name: COLUMN catch_rx.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.playtime IS 'play time (seconds)';


--
-- Name: COLUMN catch_rx.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN catch_rx.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN catch_rx.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.confidence IS 'credibility of score';


--
-- Name: COLUMN catch_rx.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.check_time IS 'last check time';


--
-- Name: COLUMN catch_rx.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.create_time IS 'submission time';


--
-- Name: COLUMN catch_rx.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.catch_rx.update_time IS 'last update time';


--
-- Name: catch_rx_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.catch_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: catch_rx_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.catch_rx_id_seq OWNED BY game_scores.catch_rx.id;


--
-- Name: mania; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN mania.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.id IS 'score''s unique id';


--
-- Name: COLUMN mania.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.user_id IS 'user''s unique id';


--
-- Name: COLUMN mania.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN mania.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.performance_v1 IS 'ppv1';


--
-- Name: COLUMN mania.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.performance_v2 IS 'ppv2';


--
-- Name: COLUMN mania.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.mods IS 'play mods';


--
-- Name: COLUMN mania.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.playtime IS 'play time (seconds)';


--
-- Name: COLUMN mania.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN mania.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN mania.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.confidence IS 'credibility of score';


--
-- Name: COLUMN mania.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.check_time IS 'last check time';


--
-- Name: COLUMN mania.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.create_time IS 'submission time';


--
-- Name: COLUMN mania.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.mania.update_time IS 'last update time';


--
-- Name: mania_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.mania_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mania_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.mania_id_seq OWNED BY game_scores.mania.id;


--
-- Name: std; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN std.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.id IS 'score''s unique id';


--
-- Name: COLUMN std.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.user_id IS 'user''s unique id';


--
-- Name: COLUMN std.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN std.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.performance_v1 IS 'ppv1';


--
-- Name: COLUMN std.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.performance_v2 IS 'ppv2';


--
-- Name: COLUMN std.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.mods IS 'play mods';


--
-- Name: COLUMN std.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.playtime IS 'play time (seconds)';


--
-- Name: COLUMN std.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN std.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN std.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.confidence IS 'credibility of score';


--
-- Name: COLUMN std.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.check_time IS 'last check time';


--
-- Name: COLUMN std.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.create_time IS 'submission time';


--
-- Name: COLUMN std.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std.update_time IS 'last update time';


--
-- Name: std_ap; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN std_ap.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.id IS 'score''s unique id';


--
-- Name: COLUMN std_ap.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.user_id IS 'user''s unique id';


--
-- Name: COLUMN std_ap.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN std_ap.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.performance_v1 IS 'ppv1';


--
-- Name: COLUMN std_ap.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.performance_v2 IS 'ppv2';


--
-- Name: COLUMN std_ap.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.mods IS 'play mods';


--
-- Name: COLUMN std_ap.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.playtime IS 'play time (seconds)';


--
-- Name: COLUMN std_ap.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN std_ap.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN std_ap.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.confidence IS 'credibility of score';


--
-- Name: COLUMN std_ap.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.check_time IS 'last check time';


--
-- Name: COLUMN std_ap.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.create_time IS 'submission time';


--
-- Name: COLUMN std_ap.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_ap.update_time IS 'last update time';


--
-- Name: std_ap_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.std_ap_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: std_ap_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.std_ap_id_seq OWNED BY game_scores.std_ap.id;


--
-- Name: std_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.std_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: std_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.std_id_seq OWNED BY game_scores.std.id;


--
-- Name: std_rx; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN std_rx.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.id IS 'score''s unique id';


--
-- Name: COLUMN std_rx.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.user_id IS 'user''s unique id';


--
-- Name: COLUMN std_rx.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN std_rx.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.performance_v1 IS 'ppv1';


--
-- Name: COLUMN std_rx.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.performance_v2 IS 'ppv2';


--
-- Name: COLUMN std_rx.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.mods IS 'play mods';


--
-- Name: COLUMN std_rx.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.playtime IS 'play time (seconds)';


--
-- Name: COLUMN std_rx.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN std_rx.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN std_rx.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.confidence IS 'credibility of score';


--
-- Name: COLUMN std_rx.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.check_time IS 'last check time';


--
-- Name: COLUMN std_rx.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.create_time IS 'submission time';


--
-- Name: COLUMN std_rx.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.std_rx.update_time IS 'last update time';


--
-- Name: std_rx_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.std_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: std_rx_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.std_rx_id_seq OWNED BY game_scores.std_rx.id;


--
-- Name: taiko; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN taiko.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.id IS 'score''s unique id';


--
-- Name: COLUMN taiko.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.user_id IS 'user''s unique id';


--
-- Name: COLUMN taiko.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN taiko.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.performance_v1 IS 'ppv1';


--
-- Name: COLUMN taiko.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.performance_v2 IS 'ppv2';


--
-- Name: COLUMN taiko.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.mods IS 'play mods';


--
-- Name: COLUMN taiko.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.playtime IS 'play time (seconds)';


--
-- Name: COLUMN taiko.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN taiko.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN taiko.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.confidence IS 'credibility of score';


--
-- Name: COLUMN taiko.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.check_time IS 'last check time';


--
-- Name: COLUMN taiko.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.create_time IS 'submission time';


--
-- Name: COLUMN taiko.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko.update_time IS 'last update time';


--
-- Name: taiko_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.taiko_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: taiko_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.taiko_id_seq OWNED BY game_scores.taiko.id;


--
-- Name: taiko_rx; Type: TABLE; Schema: game_scores; Owner: -
--

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


--
-- Name: COLUMN taiko_rx.id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.id IS 'score''s unique id';


--
-- Name: COLUMN taiko_rx.user_id; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.user_id IS 'user''s unique id';


--
-- Name: COLUMN taiko_rx.map_md5; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.map_md5 IS 'beatmap''s md5';


--
-- Name: COLUMN taiko_rx.performance_v1; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.performance_v1 IS 'ppv1';


--
-- Name: COLUMN taiko_rx.performance_v2; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.performance_v2 IS 'ppv2';


--
-- Name: COLUMN taiko_rx.mods; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.mods IS 'play mods';


--
-- Name: COLUMN taiko_rx.playtime; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.playtime IS 'play time (seconds)';


--
-- Name: COLUMN taiko_rx.perfect; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.perfect IS 'this score is full combo or not';


--
-- Name: COLUMN taiko_rx.client_version; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.client_version IS 'the client version used to submit this score';


--
-- Name: COLUMN taiko_rx.confidence; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.confidence IS 'credibility of score';


--
-- Name: COLUMN taiko_rx.check_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.check_time IS 'last check time';


--
-- Name: COLUMN taiko_rx.create_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.create_time IS 'submission time';


--
-- Name: COLUMN taiko_rx.update_time; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON COLUMN game_scores.taiko_rx.update_time IS 'last update time';


--
-- Name: taiko_rx_id_seq; Type: SEQUENCE; Schema: game_scores; Owner: -
--

CREATE SEQUENCE game_scores.taiko_rx_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: taiko_rx_id_seq; Type: SEQUENCE OWNED BY; Schema: game_scores; Owner: -
--

ALTER SEQUENCE game_scores.taiko_rx_id_seq OWNED BY game_scores.taiko_rx.id;


--
-- Name: catch; Type: TABLE; Schema: game_stats; Owner: -
--

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


--
-- Name: COLUMN catch.id; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON COLUMN game_stats.catch.id IS 'user''s unique id';


--
-- Name: mania; Type: TABLE; Schema: game_stats; Owner: -
--

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


--
-- Name: COLUMN mania.id; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON COLUMN game_stats.mania.id IS 'user''s unique id';


--
-- Name: std; Type: TABLE; Schema: game_stats; Owner: -
--

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


--
-- Name: COLUMN std.id; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON COLUMN game_stats.std.id IS 'user''s unique id';


--
-- Name: taiko; Type: TABLE; Schema: game_stats; Owner: -
--

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


--
-- Name: COLUMN taiko.id; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON COLUMN game_stats.taiko.id IS 'user''s unique id';


--
-- Name: db_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.db_versions (
    version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    author character varying(255) DEFAULT 'PurePeace'::character varying NOT NULL,
    sql text,
    release_note text,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: COLUMN db_versions.version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.db_versions.version IS 'peace database version';


--
-- Name: COLUMN db_versions.author; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.db_versions.author IS 'version publisher';


--
-- Name: COLUMN db_versions.sql; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.db_versions.sql IS 'database initial sql';


--
-- Name: COLUMN db_versions.release_note; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.db_versions.release_note IS 'version release note';


--
-- Name: versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.versions (
    version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    author character varying(255) DEFAULT 'PurePeace'::character varying NOT NULL,
    db_version character varying(15) DEFAULT '0.1.0'::character varying NOT NULL,
    release_note text,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: COLUMN versions.version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.versions.version IS 'peace version';


--
-- Name: COLUMN versions.author; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.versions.author IS 'version publisher';


--
-- Name: COLUMN versions.db_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.versions.db_version IS 'peace ''s database version';


--
-- Name: COLUMN versions.release_note; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.versions.release_note IS 'version release note';


--
-- Name: address; Type: TABLE; Schema: user; Owner: -
--

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


--
-- Name: TABLE address; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TABLE "user".address IS 'User''s login hardware address';


--
-- Name: COLUMN address.id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.id IS 'address id, unique';


--
-- Name: COLUMN address.user_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.user_id IS 'user_id, int 32';


--
-- Name: COLUMN address.time_offset; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.time_offset IS 'time_offset';


--
-- Name: COLUMN address.path; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.path IS 'osu_path hash';


--
-- Name: COLUMN address.adapters; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.adapters IS 'network physical addresses delimited by ''.''';


--
-- Name: COLUMN address.adapters_hash; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.adapters_hash IS 'adapters_hash';


--
-- Name: COLUMN address.uninstall_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.uninstall_id IS 'uniqueid1';


--
-- Name: COLUMN address.disk_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.disk_id IS 'uniqueid2';


--
-- Name: COLUMN address.create_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".address.create_time IS 'create_time';


--
-- Name: address_id_seq; Type: SEQUENCE; Schema: user; Owner: -
--

CREATE SEQUENCE "user".address_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: address_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: -
--

ALTER SEQUENCE "user".address_id_seq OWNED BY "user".address.id;


--
-- Name: base; Type: TABLE; Schema: user; Owner: -
--

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


--
-- Name: TABLE base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TABLE "user".base IS 'Basic user information, such as user name, password, email, etc.';


--
-- Name: COLUMN base.id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.id IS 'user_id, int 32, unique';


--
-- Name: COLUMN base.name; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.name IS 'username (unsafe), string, unique';


--
-- Name: COLUMN base.name_safe; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.name_safe IS 'username (safe), string, unique';


--
-- Name: COLUMN base.password; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.password IS 'user password';


--
-- Name: COLUMN base.email; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.email IS 'email, string, unique';


--
-- Name: COLUMN base.privileges; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.privileges IS 'user privileges';


--
-- Name: COLUMN base.country; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.country IS 'user country';


--
-- Name: COLUMN base.create_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.create_time IS 'user create time, auto create';


--
-- Name: COLUMN base.update_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".base.update_time IS 'user info last update time, auto create and update';


--
-- Name: base_id_seq; Type: SEQUENCE; Schema: user; Owner: -
--

CREATE SEQUENCE "user".base_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


--
-- Name: base_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: -
--

ALTER SEQUENCE "user".base_id_seq OWNED BY "user".base.id;


--
-- Name: friends; Type: TABLE; Schema: user; Owner: -
--

CREATE TABLE "user".friends (
    user_id integer NOT NULL,
    friend_id integer NOT NULL,
    remark character varying(255),
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: TABLE friends; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TABLE "user".friends IS 'User’s friends';


--
-- Name: COLUMN friends.user_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".friends.user_id IS 'user_id, int 32';


--
-- Name: COLUMN friends.friend_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".friends.friend_id IS 'user_id, int 32';


--
-- Name: COLUMN friends.remark; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".friends.remark IS 'friend remark, such as aka';


--
-- Name: COLUMN friends.create_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".friends.create_time IS 'create timestamp, auto';


--
-- Name: notes; Type: TABLE; Schema: user; Owner: -
--

CREATE TABLE "user".notes (
    id integer NOT NULL,
    user_id integer NOT NULL,
    note text NOT NULL,
    type integer DEFAULT 0 NOT NULL,
    added_by integer,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: TABLE notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TABLE "user".notes IS 'User’s notes, which may be rewards or warnings etc.';


--
-- Name: COLUMN notes.id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.id IS 'note id, unique';


--
-- Name: COLUMN notes.user_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.user_id IS 'user_id, int 32';


--
-- Name: COLUMN notes.note; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.note IS 'note, string';


--
-- Name: COLUMN notes.type; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.type IS 'note type, 0: common, 1: reward, 2: warn, 3: punish, 4: multiple accounts, 5: cheats, 6: not important';


--
-- Name: COLUMN notes.added_by; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.added_by IS 'added by who, user_id or null';


--
-- Name: COLUMN notes.create_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.create_time IS 'note create time, auto create';


--
-- Name: COLUMN notes.update_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".notes.update_time IS 'note last update time, auto create and update';


--
-- Name: notes_id_seq; Type: SEQUENCE; Schema: user; Owner: -
--

CREATE SEQUENCE "user".notes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


--
-- Name: notes_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: -
--

ALTER SEQUENCE "user".notes_id_seq OWNED BY "user".notes.id;


--
-- Name: statistic; Type: TABLE; Schema: user; Owner: -
--

CREATE TABLE "user".statistic (
    id integer NOT NULL,
    online_duration interval DEFAULT '00:00:00'::interval NOT NULL,
    login_count integer DEFAULT 0 NOT NULL,
    rename_count integer DEFAULT 0 NOT NULL,
    friends_count integer DEFAULT 0 NOT NULL,
    notes_count integer DEFAULT 0 NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: COLUMN statistic.id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.id IS 'user''s unique id';


--
-- Name: COLUMN statistic.online_duration; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.online_duration IS 'user''s total online duration';


--
-- Name: COLUMN statistic.login_count; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.login_count IS 'user''s total login count';


--
-- Name: COLUMN statistic.rename_count; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.rename_count IS 'user''s total rename count';


--
-- Name: COLUMN statistic.friends_count; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.friends_count IS 'user''s total friend count';


--
-- Name: COLUMN statistic.notes_count; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.notes_count IS 'user''s total note count';


--
-- Name: COLUMN statistic.update_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".statistic.update_time IS 'update time';


--
-- Name: login; Type: TABLE; Schema: user_records; Owner: -
--

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


--
-- Name: TABLE login; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON TABLE user_records.login IS 'The user''s login record, associated with the user''s login address';


--
-- Name: COLUMN login.id; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.id IS 'login record id';


--
-- Name: COLUMN login.user_id; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.user_id IS 'user.id, int 32';


--
-- Name: COLUMN login.address_id; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.address_id IS 'user.address.id';


--
-- Name: COLUMN login.ip; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.ip IS 'ip address';


--
-- Name: COLUMN login.version; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.version IS 'osu version';


--
-- Name: COLUMN login.similarity; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.similarity IS 'certainty of the address';


--
-- Name: COLUMN login.create_time; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.create_time IS 'create_time, auto';


--
-- Name: COLUMN login.logout_time; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.logout_time IS 'this record''s logout time';


--
-- Name: COLUMN login.online_duration; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.login.online_duration IS 'online duration';


--
-- Name: login_records_id_seq; Type: SEQUENCE; Schema: user_records; Owner: -
--

CREATE SEQUENCE user_records.login_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: login_records_id_seq; Type: SEQUENCE OWNED BY; Schema: user_records; Owner: -
--

ALTER SEQUENCE user_records.login_records_id_seq OWNED BY user_records.login.id;


--
-- Name: rename; Type: TABLE; Schema: user_records; Owner: -
--

CREATE TABLE user_records.rename (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    new_name character varying(255) NOT NULL,
    old_name character varying(255) NOT NULL,
    create_time timestamp(0) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: TABLE rename; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON TABLE user_records.rename IS 'Automatically record the user''s rename record (do not add manually)';


--
-- Name: COLUMN rename.id; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.rename.id IS 'rename records id';


--
-- Name: COLUMN rename.user_id; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.rename.user_id IS 'user''s unique id';


--
-- Name: COLUMN rename.new_name; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.rename.new_name IS 'user''s new name (after rename)';


--
-- Name: COLUMN rename.old_name; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.rename.old_name IS 'user''s old name (before rename)';


--
-- Name: COLUMN rename.create_time; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON COLUMN user_records.rename.create_time IS 'rename records create time';


--
-- Name: rename_records_id_seq; Type: SEQUENCE; Schema: user_records; Owner: -
--

CREATE SEQUENCE user_records.rename_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


--
-- Name: rename_records_id_seq; Type: SEQUENCE OWNED BY; Schema: user_records; Owner: -
--

ALTER SEQUENCE user_records.rename_records_id_seq OWNED BY user_records.rename.id;


--
-- Name: channels id; Type: DEFAULT; Schema: bancho; Owner: -
--

ALTER TABLE ONLY bancho.channels ALTER COLUMN id SET DEFAULT nextval('bancho.channels_id_seq'::regclass);


--
-- Name: catch id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.catch ALTER COLUMN id SET DEFAULT nextval('game_scores.catch_id_seq'::regclass);


--
-- Name: catch_rx id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.catch_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.catch_rx_id_seq'::regclass);


--
-- Name: mania id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.mania ALTER COLUMN id SET DEFAULT nextval('game_scores.mania_id_seq'::regclass);


--
-- Name: std id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std ALTER COLUMN id SET DEFAULT nextval('game_scores.std_id_seq'::regclass);


--
-- Name: std_ap id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std_ap ALTER COLUMN id SET DEFAULT nextval('game_scores.std_ap_id_seq'::regclass);


--
-- Name: std_rx id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.std_rx_id_seq'::regclass);


--
-- Name: taiko id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.taiko ALTER COLUMN id SET DEFAULT nextval('game_scores.taiko_id_seq'::regclass);


--
-- Name: taiko_rx id; Type: DEFAULT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.taiko_rx ALTER COLUMN id SET DEFAULT nextval('game_scores.taiko_rx_id_seq'::regclass);


--
-- Name: address id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".address ALTER COLUMN id SET DEFAULT nextval('"user".address_id_seq'::regclass);


--
-- Name: base id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base ALTER COLUMN id SET DEFAULT nextval('"user".base_id_seq'::regclass);


--
-- Name: notes id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes ALTER COLUMN id SET DEFAULT nextval('"user".notes_id_seq'::regclass);


--
-- Name: login id; Type: DEFAULT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.login ALTER COLUMN id SET DEFAULT nextval('user_records.login_records_id_seq'::regclass);


--
-- Name: rename id; Type: DEFAULT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.rename ALTER COLUMN id SET DEFAULT nextval('user_records.rename_records_id_seq'::regclass);


--
-- Data for Name: channels; Type: TABLE DATA; Schema: bancho; Owner: -
--

INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (1, '#osu', 'General discussion.', 1, 2, true, '2020-12-09 04:21:05.471552+08', '2020-12-09 04:21:14.652774+08');
INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (3, '#announce', 'Exemplary performance and public announcements.', 1, 2, true, '2020-12-09 04:21:35.551317+08', '2020-12-09 04:21:35.551317+08');
INSERT INTO bancho.channels (id, name, title, read_priv, write_priv, auto_join, create_time, update_time) VALUES (4, '#lobby', 'Multiplayer lobby discussion room.', 1, 2, true, '2020-12-09 04:21:46.339821+08', '2020-12-09 04:21:46.339821+08');


--
-- Data for Name: catch; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: catch_rx; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: mania; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: std; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: std_ap; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: std_rx; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: taiko; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: taiko_rx; Type: TABLE DATA; Schema: game_scores; Owner: -
--



--
-- Data for Name: catch; Type: TABLE DATA; Schema: game_stats; Owner: -
--

INSERT INTO game_stats.catch (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.catch (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');


--
-- Data for Name: mania; Type: TABLE DATA; Schema: game_stats; Owner: -
--

INSERT INTO game_stats.mania (id, total_score, ranked_score, performance_v1, performance_v2, playcount, total_hits, accuracy, maxcombo, playtime, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.mania (id, total_score, ranked_score, performance_v1, performance_v2, playcount, total_hits, accuracy, maxcombo, playtime, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');


--
-- Data for Name: std; Type: TABLE DATA; Schema: game_stats; Owner: -
--

INSERT INTO game_stats.std (id, total_score, ranked_score, total_score_rx, ranked_score_rx, total_score_ap, ranked_score_ap, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, performance_v1_ap, performance_v2_ap, playcount, playcount_rx, playcount_ap, total_hits, total_hits_rx, total_hits_ap, accuracy, accuracy_rx, accuracy_ap, maxcombo, maxcombo_rx, maxcombo_ap, playtime, playtime_rx, playtime_ap, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.std (id, total_score, ranked_score, total_score_rx, ranked_score_rx, total_score_ap, ranked_score_ap, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, performance_v1_ap, performance_v2_ap, playcount, playcount_rx, playcount_ap, total_hits, total_hits_rx, total_hits_ap, accuracy, accuracy_rx, accuracy_ap, maxcombo, maxcombo_rx, maxcombo_ap, playtime, playtime_rx, playtime_ap, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');


--
-- Data for Name: taiko; Type: TABLE DATA; Schema: game_stats; Owner: -
--

INSERT INTO game_stats.taiko (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:39.815269+08');
INSERT INTO game_stats.taiko (id, total_score, ranked_score, total_score_rx, ranked_score_rx, performance_v1, performance_v2, performance_v1_rx, performance_v2_rx, playcount, playcount_rx, total_hits, total_hits_rx, accuracy, accuracy_rx, maxcombo, maxcombo_rx, playtime, playtime_rx, update_time) VALUES (2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');


--
-- Data for Name: db_versions; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.db_versions (version, author, sql, release_note, create_time, update_time) VALUES ('0.1.0', 'PurePeace', NULL, 'initial', '2020-12-15 01:15:37.586205+08', '2020-12-20 01:13:47.84393+08');
INSERT INTO public.db_versions (version, author, sql, release_note, create_time, update_time) VALUES ('0.1.3', 'PurePeace', NULL, 'add game_scores, game_stats, modify some column', '2020-12-15 01:15:37.586205+08', '2020-12-15 01:15:52.635208+08');


--
-- Data for Name: versions; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.versions (version, author, db_version, release_note, create_time, update_time) VALUES ('0.1.0', 'PurePeace', '0.1.0', 'initial (wip)', '2020-12-15 01:16:37.785543+08', '2020-12-20 01:16:34.355013+08');
INSERT INTO public.versions (version, author, db_version, release_note, create_time, update_time) VALUES ('0.1.2', 'PurePeace', '0.1.3', 'add tables', '2020-12-15 01:16:37.785543+08', '2020-12-20 01:16:34.355013+08');


--
-- Data for Name: address; Type: TABLE DATA; Schema: user; Owner: -
--



--
-- Data for Name: base; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (2, 'ChinoChan', 'chinochan', '931ffe4c39bc9fdc875cf8f691bf1f57', 'a@chino.com', 1, 'JP', '2020-12-19 21:35:54.465545+08', '2020-12-20 01:12:42.56606+08');
INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (1, 'PurePeace', 'purepeace', '931ffe4c39bc9fdc875cf8f691bf1f57', '940857703@qq.com', 3, 'CN', '2020-12-19 21:35:32.810099+08', '2020-12-20 01:18:58.947387+08');


--
-- Data for Name: friends; Type: TABLE DATA; Schema: user; Owner: -
--



--
-- Data for Name: notes; Type: TABLE DATA; Schema: user; Owner: -
--



--
-- Data for Name: statistic; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".statistic (id, online_duration, login_count, rename_count, friends_count, notes_count, update_time) VALUES (2, '00:00:00', 0, 0, 0, 0, '2020-12-20 01:12:42.56606+08');
INSERT INTO "user".statistic (id, online_duration, login_count, rename_count, friends_count, notes_count, update_time) VALUES (1, '00:00:00', 0, 0, 0, 0, '2020-12-20 01:23:57.20465+08');


--
-- Data for Name: login; Type: TABLE DATA; Schema: user_records; Owner: -
--



--
-- Data for Name: rename; Type: TABLE DATA; Schema: user_records; Owner: -
--



--
-- Name: channels_id_seq; Type: SEQUENCE SET; Schema: bancho; Owner: -
--

SELECT pg_catalog.setval('bancho.channels_id_seq', 4, true);


--
-- Name: catch_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.catch_id_seq', 1, false);


--
-- Name: catch_rx_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.catch_rx_id_seq', 1, false);


--
-- Name: mania_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.mania_id_seq', 1, false);


--
-- Name: std_ap_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.std_ap_id_seq', 1, false);


--
-- Name: std_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.std_id_seq', 1, false);


--
-- Name: std_rx_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.std_rx_id_seq', 1, false);


--
-- Name: taiko_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.taiko_id_seq', 1, false);


--
-- Name: taiko_rx_id_seq; Type: SEQUENCE SET; Schema: game_scores; Owner: -
--

SELECT pg_catalog.setval('game_scores.taiko_rx_id_seq', 1, false);


--
-- Name: address_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".address_id_seq', 1, true);


--
-- Name: base_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".base_id_seq', 100, true);


--
-- Name: notes_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".notes_id_seq', 1, true);


--
-- Name: login_records_id_seq; Type: SEQUENCE SET; Schema: user_records; Owner: -
--

SELECT pg_catalog.setval('user_records.login_records_id_seq', 1, true);


--
-- Name: rename_records_id_seq; Type: SEQUENCE SET; Schema: user_records; Owner: -
--

SELECT pg_catalog.setval('user_records.rename_records_id_seq', 1, true);


--
-- Name: channels channel.name; Type: CONSTRAINT; Schema: bancho; Owner: -
--

ALTER TABLE ONLY bancho.channels
    ADD CONSTRAINT "channel.name" UNIQUE (name);


--
-- Name: CONSTRAINT "channel.name" ON channels; Type: COMMENT; Schema: bancho; Owner: -
--

COMMENT ON CONSTRAINT "channel.name" ON bancho.channels IS 'channel name should be unique';


--
-- Name: channels channels_pkey; Type: CONSTRAINT; Schema: bancho; Owner: -
--

ALTER TABLE ONLY bancho.channels
    ADD CONSTRAINT channels_pkey PRIMARY KEY (id);


--
-- Name: catch_rx catch_copy1_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.catch_rx
    ADD CONSTRAINT catch_copy1_pkey PRIMARY KEY (id);


--
-- Name: mania std_copy1_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.mania
    ADD CONSTRAINT std_copy1_pkey PRIMARY KEY (id);


--
-- Name: std_rx std_copy1_pkey1; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std_rx
    ADD CONSTRAINT std_copy1_pkey1 PRIMARY KEY (id);


--
-- Name: taiko std_copy2_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.taiko
    ADD CONSTRAINT std_copy2_pkey PRIMARY KEY (id);


--
-- Name: std_ap std_copy2_pkey1; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std_ap
    ADD CONSTRAINT std_copy2_pkey1 PRIMARY KEY (id);


--
-- Name: catch std_copy3_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.catch
    ADD CONSTRAINT std_copy3_pkey PRIMARY KEY (id);


--
-- Name: std std_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.std
    ADD CONSTRAINT std_pkey PRIMARY KEY (id);


--
-- Name: taiko_rx taiko_copy1_pkey; Type: CONSTRAINT; Schema: game_scores; Owner: -
--

ALTER TABLE ONLY game_scores.taiko_rx
    ADD CONSTRAINT taiko_copy1_pkey PRIMARY KEY (id);


--
-- Name: mania catch_copy1_pkey; Type: CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.mania
    ADD CONSTRAINT catch_copy1_pkey PRIMARY KEY (id);


--
-- Name: catch std_copy1_pkey; Type: CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.catch
    ADD CONSTRAINT std_copy1_pkey PRIMARY KEY (id);


--
-- Name: taiko std_copy1_pkey1; Type: CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.taiko
    ADD CONSTRAINT std_copy1_pkey1 PRIMARY KEY (id);


--
-- Name: std std_pkey; Type: CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.std
    ADD CONSTRAINT std_pkey PRIMARY KEY (id);


--
-- Name: versions config_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.versions
    ADD CONSTRAINT config_pkey PRIMARY KEY (version);


--
-- Name: db_versions versions_copy1_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.db_versions
    ADD CONSTRAINT versions_copy1_pkey PRIMARY KEY (version);


--
-- Name: notes Note.id; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "Note.id" UNIQUE (id);


--
-- Name: CONSTRAINT "Note.id" ON notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "Note.id" ON "user".notes IS 'note id should be unique';


--
-- Name: base Unique - email; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - email" UNIQUE (email);


--
-- Name: CONSTRAINT "Unique - email" ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "Unique - email" ON "user".base IS 'email should be unique';


--
-- Name: base Unique - name; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - name" UNIQUE (name);


--
-- Name: base Unique - name safe; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - name safe" UNIQUE (name_safe);


--
-- Name: CONSTRAINT "Unique - name safe" ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "Unique - name safe" ON "user".base IS 'name safe should be unique';


--
-- Name: address address_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT address_pkey PRIMARY KEY (id);


--
-- Name: base base_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT base_pkey PRIMARY KEY (id);


--
-- Name: friends friends_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT friends_pkey PRIMARY KEY (user_id, friend_id);


--
-- Name: notes notes_pkey1; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT notes_pkey1 PRIMARY KEY (id, user_id);


--
-- Name: statistic statistic_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".statistic
    ADD CONSTRAINT statistic_pkey PRIMARY KEY (id);


--
-- Name: login login_records_pkey; Type: CONSTRAINT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT login_records_pkey PRIMARY KEY (id);


--
-- Name: rename rename_records_pkey; Type: CONSTRAINT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.rename
    ADD CONSTRAINT rename_records_pkey PRIMARY KEY (id);


--
-- Name: User.name; Type: INDEX; Schema: user; Owner: -
--

CREATE UNIQUE INDEX "User.name" ON "user".base USING btree (name, name_safe);


--
-- Name: user_address; Type: INDEX; Schema: user; Owner: -
--

CREATE INDEX user_address ON "user".address USING btree (user_id);


--
-- Name: channels auto_update_time; Type: TRIGGER; Schema: bancho; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON bancho.channels FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: catch auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.catch FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON catch; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.catch IS 'auto update time';


--
-- Name: catch_rx auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.catch_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON catch_rx; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.catch_rx IS 'auto update time';


--
-- Name: mania auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.mania FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON mania; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.mania IS 'auto update time';


--
-- Name: std auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON std; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.std IS 'auto update time';


--
-- Name: std_ap auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std_ap FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON std_ap; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.std_ap IS 'auto update time';


--
-- Name: std_rx auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.std_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON std_rx; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.std_rx IS 'auto update time';


--
-- Name: taiko auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.taiko FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON taiko; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.taiko IS 'auto update time';


--
-- Name: taiko_rx auto_update_time; Type: TRIGGER; Schema: game_scores; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_scores.taiko_rx FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON taiko_rx; Type: COMMENT; Schema: game_scores; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_scores.taiko_rx IS 'auto update time';


--
-- Name: catch auto_update_time; Type: TRIGGER; Schema: game_stats; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.catch FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON catch; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_stats.catch IS 'auto update the time';


--
-- Name: mania auto_update_time; Type: TRIGGER; Schema: game_stats; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.mania FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON mania; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_stats.mania IS 'auto update the time';


--
-- Name: std auto_update_time; Type: TRIGGER; Schema: game_stats; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.std FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON std; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_stats.std IS 'auto update the time';


--
-- Name: taiko auto_update_time; Type: TRIGGER; Schema: game_stats; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON game_stats.taiko FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON taiko; Type: COMMENT; Schema: game_stats; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON game_stats.taiko IS 'auto update the time';


--
-- Name: db_versions auto_update_time; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON public.db_versions FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: versions auto_update_time; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON public.versions FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: base auto_insert_related; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER auto_insert_related AFTER INSERT ON "user".base FOR EACH ROW EXECUTE PROCEDURE "user".insert_related_on_base_insert();


--
-- Name: TRIGGER auto_insert_related ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER auto_insert_related ON "user".base IS 'auto insert into related table';


--
-- Name: statistic auto_update_time; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER auto_update_time BEFORE UPDATE ON "user".statistic FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_time ON statistic; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER auto_update_time ON "user".statistic IS 'auto update the timestamp';


--
-- Name: base auto_update_timestamp; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER auto_update_timestamp BEFORE UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER auto_update_timestamp ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER auto_update_timestamp ON "user".base IS 'auto update the update_time after update user info';


--
-- Name: friends decrease_friend_count; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER decrease_friend_count AFTER DELETE ON "user".friends FOR EACH ROW EXECUTE PROCEDURE "user".decrease_friend_count();


--
-- Name: TRIGGER decrease_friend_count ON friends; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER decrease_friend_count ON "user".friends IS 'update the statistic';


--
-- Name: notes decrease_note_count; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER decrease_note_count AFTER DELETE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE "user".decrease_note_count();


--
-- Name: TRIGGER decrease_note_count ON notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER decrease_note_count ON "user".notes IS 'update the statistic';


--
-- Name: friends increase_friend_count; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER increase_friend_count AFTER INSERT ON "user".friends FOR EACH ROW EXECUTE PROCEDURE "user".increase_friend_count();


--
-- Name: TRIGGER increase_friend_count ON friends; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER increase_friend_count ON "user".friends IS 'update the statistic';


--
-- Name: notes increase_note_count; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER increase_note_count AFTER INSERT ON "user".notes FOR EACH ROW EXECUTE PROCEDURE "user".increase_note_count();


--
-- Name: TRIGGER increase_note_count ON notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER increase_note_count ON "user".notes IS 'update the statistic';


--
-- Name: base safe_user_info; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER safe_user_info BEFORE INSERT OR UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE "user".safe_user_info();


--
-- Name: TRIGGER safe_user_info ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER safe_user_info ON "user".base IS 'auto make the user info safety';


--
-- Name: notes update_time_auto; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER update_time_auto ON "user".notes IS 'auto update the update_time after update note info';


--
-- Name: login auto_login_duration; Type: TRIGGER; Schema: user_records; Owner: -
--

CREATE TRIGGER auto_login_duration BEFORE UPDATE ON user_records.login FOR EACH ROW EXECUTE PROCEDURE user_records.auto_online_duration();


--
-- Name: TRIGGER auto_login_duration ON login; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON TRIGGER auto_login_duration ON user_records.login IS 'auto update the online duration';


--
-- Name: login increase_login_count; Type: TRIGGER; Schema: user_records; Owner: -
--

CREATE TRIGGER increase_login_count BEFORE INSERT ON user_records.login FOR EACH ROW EXECUTE PROCEDURE user_records.increase_login_count();


--
-- Name: TRIGGER increase_login_count ON login; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON TRIGGER increase_login_count ON user_records.login IS 'auto update the statistic';


--
-- Name: rename increase_rename_count; Type: TRIGGER; Schema: user_records; Owner: -
--

CREATE TRIGGER increase_rename_count BEFORE INSERT ON user_records.rename FOR EACH ROW EXECUTE PROCEDURE user_records.increase_rename_count();


--
-- Name: TRIGGER increase_rename_count ON rename; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON TRIGGER increase_rename_count ON user_records.rename IS 'update user statistic';


--
-- Name: catch user.id; Type: FK CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.catch
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: mania user.id; Type: FK CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.mania
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: std user.id; Type: FK CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.std
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: taiko user.id; Type: FK CONSTRAINT; Schema: game_stats; Owner: -
--

ALTER TABLE ONLY game_stats.taiko
    ADD CONSTRAINT "user.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: versions db_version; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.versions
    ADD CONSTRAINT db_version FOREIGN KEY (db_version) REFERENCES public.db_versions(version) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: friends User.id; Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id" ON friends; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "User.id" ON "user".friends IS 'user_id';


--
-- Name: notes User.id; Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: address User.id; Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: statistic User.id; Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".statistic
    ADD CONSTRAINT "User.id" FOREIGN KEY (id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id" ON statistic; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "User.id" ON "user".statistic IS 'user''s unique id';


--
-- Name: notes User.id (added_by); Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id (added_by)" FOREIGN KEY (added_by) REFERENCES "user".base(id) ON UPDATE CASCADE;


--
-- Name: friends User.id (friend); Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id (friend)" FOREIGN KEY (friend_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id (friend)" ON friends; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "User.id (friend)" ON "user".friends IS 'user_id (friend)';


--
-- Name: rename User.id; Type: FK CONSTRAINT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.rename
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id" ON rename; Type: COMMENT; Schema: user_records; Owner: -
--

COMMENT ON CONSTRAINT "User.id" ON user_records.rename IS 'user''s unique id';


--
-- Name: login User.id; Type: FK CONSTRAINT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: login address.id; Type: FK CONSTRAINT; Schema: user_records; Owner: -
--

ALTER TABLE ONLY user_records.login
    ADD CONSTRAINT "address.id" FOREIGN KEY (address_id) REFERENCES "user".address(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

