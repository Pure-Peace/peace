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
-- Name: user; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA "user";


--
-- Name: update_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN
	new.update_time = CURRENT_TIMESTAMP;
	RETURN new;
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
		NEW.name_safe = REPLACE(LOWER(NEW.name), ' ', '_');
	RETURN NEW;
END$$;


SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: address; Type: TABLE; Schema: user; Owner: -
--

CREATE TABLE "user".address (
    id integer NOT NULL,
    user_id integer NOT NULL,
    time_offset character varying(255) NOT NULL,
    path character varying(255) NOT NULL,
    adapters text NOT NULL,
    adapters_hash character varying(255) NOT NULL,
    uninstall_id character varying NOT NULL,
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
-- Name: login_records; Type: TABLE; Schema: user; Owner: -
--

CREATE TABLE "user".login_records (
    id integer NOT NULL,
    user_id integer NOT NULL,
    address_id integer NOT NULL,
    ip character varying(255) NOT NULL,
    version character varying(255) NOT NULL,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: TABLE login_records; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TABLE "user".login_records IS 'The user''s login record, associated with the user''s login address';


--
-- Name: COLUMN login_records.id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.id IS 'login record id';


--
-- Name: COLUMN login_records.user_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.user_id IS 'user.id, int 32';


--
-- Name: COLUMN login_records.address_id; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.address_id IS 'user.address.id';


--
-- Name: COLUMN login_records.ip; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.ip IS 'ip address';


--
-- Name: COLUMN login_records.version; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.version IS 'osu version';


--
-- Name: COLUMN login_records.create_time; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON COLUMN "user".login_records.create_time IS 'create_time, auto';


--
-- Name: login_records_id_seq; Type: SEQUENCE; Schema: user; Owner: -
--

CREATE SEQUENCE "user".login_records_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: login_records_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: -
--

ALTER SEQUENCE "user".login_records_id_seq OWNED BY "user".login_records.id;


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
-- Name: address id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".address ALTER COLUMN id SET DEFAULT nextval('"user".address_id_seq'::regclass);


--
-- Name: base id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base ALTER COLUMN id SET DEFAULT nextval('"user".base_id_seq'::regclass);


--
-- Name: login_records id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".login_records ALTER COLUMN id SET DEFAULT nextval('"user".login_records_id_seq'::regclass);


--
-- Name: notes id; Type: DEFAULT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes ALTER COLUMN id SET DEFAULT nextval('"user".notes_id_seq'::regclass);


--
-- Data for Name: address; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".address (id, user_id, time_offset, path, adapters, adapters_hash, uninstall_id, disk_id, create_time) VALUES (1, 1009, '1', 'a', 'a', 'a', 'a', 'a', '2020-11-22 00:54:58.723191+08');
INSERT INTO "user".address (id, user_id, time_offset, path, adapters, adapters_hash, uninstall_id, disk_id, create_time) VALUES (2, 1009, '2', 'b', 'b', 'b', 'b', 'b', '2020-11-22 00:55:05.122778+08');
INSERT INTO "user".address (id, user_id, time_offset, path, adapters, adapters_hash, uninstall_id, disk_id, create_time) VALUES (3, 1011, '3', 'c', 'c', 'c', 'c', 'c', '2020-11-22 00:55:12.680359+08');


--
-- Data for Name: base; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (1009, 'PurePeace', 'purepeace', '931ffe4c39bc9fdc875cf8f691bf1f57', '940857703@qq.com', 1, 'CN', '2020-11-21 23:42:00.487276+08', '2020-11-21 23:42:15.498228+08');
INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (1011, 'Chino', 'chino', '931ffe4c39bc9fdc875cf8f691bf1f57', 'chino@kafuu.com', 1, 'UN', '2020-11-21 23:43:18.460883+08', '2020-11-21 23:43:18.460883+08');
INSERT INTO "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) VALUES (1012, 'usao', 'usao', '931ffe4c39bc9fdc875cf8f691bf1f57', '1', 1, 'UN', '2020-11-21 23:43:32.801019+08', '2020-11-21 23:43:32.801019+08');


--
-- Data for Name: friends; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".friends (user_id, friend_id, remark, create_time) VALUES (1009, 1011, NULL, '2020-11-21 23:45:28.794136+08');
INSERT INTO "user".friends (user_id, friend_id, remark, create_time) VALUES (1009, 1012, NULL, '2020-11-21 23:45:37.559363+08');
INSERT INTO "user".friends (user_id, friend_id, remark, create_time) VALUES (1011, 1009, NULL, '2020-11-21 23:45:47.362144+08');


--
-- Data for Name: login_records; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".login_records (id, user_id, address_id, ip, version, create_time) VALUES (3, 1009, 1, 'a', 'a', '2020-11-22 00:55:26.9644+08');
INSERT INTO "user".login_records (id, user_id, address_id, ip, version, create_time) VALUES (4, 1009, 1, 'f', 'f', '2020-11-22 00:55:36.010122+08');
INSERT INTO "user".login_records (id, user_id, address_id, ip, version, create_time) VALUES (5, 1009, 2, 'c', 'c', '2020-11-22 00:55:42.297614+08');
INSERT INTO "user".login_records (id, user_id, address_id, ip, version, create_time) VALUES (6, 1011, 3, 'g', 'g', '2020-11-22 00:55:50.493531+08');


--
-- Data for Name: notes; Type: TABLE DATA; Schema: user; Owner: -
--

INSERT INTO "user".notes (id, user_id, note, type, added_by, create_time, update_time) VALUES (1, 1009, 'boss', 0, NULL, '2020-11-21 23:46:12.296661+08', '2020-11-21 23:46:12.296661+08');


--
-- Name: address_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".address_id_seq', 3, true);


--
-- Name: base_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".base_id_seq', 1012, true);


--
-- Name: login_records_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".login_records_id_seq', 6, true);


--
-- Name: notes_id_seq; Type: SEQUENCE SET; Schema: user; Owner: -
--

SELECT pg_catalog.setval('"user".notes_id_seq', 1, true);


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
-- Name: base Unique - some user info; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - some user info" UNIQUE (id, name, name_safe, email);


--
-- Name: CONSTRAINT "Unique - some user info" ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON CONSTRAINT "Unique - some user info" ON "user".base IS 'id, name, name_safe, email should be unique';


--
-- Name: address address_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT address_pkey PRIMARY KEY (id, user_id);


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
-- Name: login_records login_records_id; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT login_records_id UNIQUE (id);


--
-- Name: login_records login_records_pkey; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT login_records_pkey PRIMARY KEY (id, user_id);


--
-- Name: notes notes_pkey1; Type: CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT notes_pkey1 PRIMARY KEY (id, user_id);


--
-- Name: User.name; Type: INDEX; Schema: user; Owner: -
--

CREATE UNIQUE INDEX "User.name" ON "user".base USING btree (name, name_safe);


--
-- Name: user_address; Type: INDEX; Schema: user; Owner: -
--

CREATE INDEX user_address ON "user".address USING btree (user_id);


--
-- Name: base safe_user_info; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER safe_user_info BEFORE INSERT OR UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE "user".safe_user_info();


--
-- Name: TRIGGER safe_user_info ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER safe_user_info ON "user".base IS 'auto make the user info safety';


--
-- Name: base update_time_auto; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON base; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER update_time_auto ON "user".base IS 'auto update the update_time after update user info';


--
-- Name: notes update_time_auto; Type: TRIGGER; Schema: user; Owner: -
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON notes; Type: COMMENT; Schema: user; Owner: -
--

COMMENT ON TRIGGER update_time_auto ON "user".notes IS 'auto update the update_time after update note info';


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
-- Name: login_records login_records; Type: FK CONSTRAINT; Schema: user; Owner: -
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT login_records FOREIGN KEY (address_id, user_id) REFERENCES "user".address(id, user_id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

