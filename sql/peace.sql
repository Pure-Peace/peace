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
-- Name: bancho; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA bancho;


ALTER SCHEMA bancho OWNER TO postgres;

--
-- Name: user; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA "user";


ALTER SCHEMA "user" OWNER TO postgres;

--
-- Name: update_timestamp(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$BEGIN



	new.update_time = CURRENT_TIMESTAMP;



	RETURN new;



END$$;


ALTER FUNCTION public.update_timestamp() OWNER TO postgres;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: address; Type: TABLE; Schema: user; Owner: postgres
--

CREATE TABLE "user".address (
    id integer NOT NULL,
    user_id integer NOT NULL,
    ip character varying(255),
    version character varying(255),
    time_offset character varying(255),
    location character varying(255),
    path character varying(255),
    adapters text,
    adapters_hash character varying(255),
    uninstall_id character varying,
    disk_id character varying(255),
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE "user".address OWNER TO postgres;

--
-- Name: COLUMN address.id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.id IS 'address id, unique';


--
-- Name: COLUMN address.user_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.user_id IS 'user_id, int 32';


--
-- Name: COLUMN address.ip; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.ip IS 'ip address';


--
-- Name: COLUMN address.version; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.version IS 'osu_version';


--
-- Name: COLUMN address.time_offset; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.time_offset IS 'time_offset';


--
-- Name: COLUMN address.location; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.location IS 'location';


--
-- Name: COLUMN address.path; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.path IS 'osu_path hash';


--
-- Name: COLUMN address.adapters; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.adapters IS 'network physical addresses delimited by ''.''';


--
-- Name: COLUMN address.adapters_hash; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.adapters_hash IS 'adapters_hash';


--
-- Name: COLUMN address.uninstall_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.uninstall_id IS 'uniqueid1';


--
-- Name: COLUMN address.disk_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.disk_id IS 'uniqueid2';


--
-- Name: COLUMN address.create_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".address.create_time IS 'create_time';


--
-- Name: address_id_seq; Type: SEQUENCE; Schema: user; Owner: postgres
--

CREATE SEQUENCE "user".address_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


ALTER TABLE "user".address_id_seq OWNER TO postgres;

--
-- Name: address_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: postgres
--

ALTER SEQUENCE "user".address_id_seq OWNED BY "user".address.id;


--
-- Name: base; Type: TABLE; Schema: user; Owner: postgres
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


ALTER TABLE "user".base OWNER TO postgres;

--
-- Name: COLUMN base.id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.id IS 'user_id, int 32, unique';


--
-- Name: COLUMN base.name; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.name IS 'username (unsafe), string, unique';


--
-- Name: COLUMN base.name_safe; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.name_safe IS 'username (safe), string, unique';


--
-- Name: COLUMN base.password; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.password IS 'user password';


--
-- Name: COLUMN base.email; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.email IS 'email, string, unique';


--
-- Name: COLUMN base.privileges; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.privileges IS 'user privileges';


--
-- Name: COLUMN base.country; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.country IS 'user country';


--
-- Name: COLUMN base.create_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.create_time IS 'user create time, auto create';


--
-- Name: COLUMN base.update_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".base.update_time IS 'user info last update time, auto create and update';


--
-- Name: base_id_seq; Type: SEQUENCE; Schema: user; Owner: postgres
--

CREATE SEQUENCE "user".base_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


ALTER TABLE "user".base_id_seq OWNER TO postgres;

--
-- Name: base_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: postgres
--

ALTER SEQUENCE "user".base_id_seq OWNED BY "user".base.id;


--
-- Name: friends; Type: TABLE; Schema: user; Owner: postgres
--

CREATE TABLE "user".friends (
    user_id integer NOT NULL,
    friend_id integer NOT NULL,
    remark character varying(255),
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE "user".friends OWNER TO postgres;

--
-- Name: COLUMN friends.user_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".friends.user_id IS 'user_id, int 32';


--
-- Name: COLUMN friends.friend_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".friends.friend_id IS 'user_id, int 32';


--
-- Name: COLUMN friends.remark; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".friends.remark IS 'friend remark, such as aka';


--
-- Name: COLUMN friends.create_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".friends.create_time IS 'create timestamp, auto';


--
-- Name: login_records; Type: TABLE; Schema: user; Owner: postgres
--

CREATE TABLE "user".login_records (
    id integer NOT NULL,
    user_id integer NOT NULL,
    address_id integer NOT NULL,
    count integer DEFAULT 1 NOT NULL,
    timestamps character varying[] NOT NULL,
    create_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    update_time timestamp(6) with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE "user".login_records OWNER TO postgres;

--
-- Name: COLUMN login_records.id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.id IS 'login_records id, unique';


--
-- Name: COLUMN login_records.user_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.user_id IS 'user.id, int 32';


--
-- Name: COLUMN login_records.address_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.address_id IS 'user.address.id';


--
-- Name: COLUMN login_records.count; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.count IS 'address login count';


--
-- Name: COLUMN login_records.timestamps; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.timestamps IS 'each login timestamps';


--
-- Name: COLUMN login_records.create_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.create_time IS 'create_time, auto';


--
-- Name: COLUMN login_records.update_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".login_records.update_time IS 'update_time, auto';


--
-- Name: login_records_id_seq; Type: SEQUENCE; Schema: user; Owner: postgres
--

CREATE SEQUENCE "user".login_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


ALTER TABLE "user".login_records_id_seq OWNER TO postgres;

--
-- Name: login_records_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: postgres
--

ALTER SEQUENCE "user".login_records_id_seq OWNED BY "user".login_records.id;


--
-- Name: notes; Type: TABLE; Schema: user; Owner: postgres
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


ALTER TABLE "user".notes OWNER TO postgres;

--
-- Name: COLUMN notes.id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.id IS 'note id, unique';


--
-- Name: COLUMN notes.user_id; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.user_id IS 'user_id, int 32';


--
-- Name: COLUMN notes.note; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.note IS 'note, string';


--
-- Name: COLUMN notes.type; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.type IS 'note type, 0: common, 1: reward, 2: warn, 3: punish, 4: multiple accounts, 5: cheats, 6: not important';


--
-- Name: COLUMN notes.added_by; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.added_by IS 'added by who, user_id or null';


--
-- Name: COLUMN notes.create_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.create_time IS 'note create time, auto create';


--
-- Name: COLUMN notes.update_time; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON COLUMN "user".notes.update_time IS 'note last update time, auto create and update';


--
-- Name: notes_id_seq; Type: SEQUENCE; Schema: user; Owner: postgres
--

CREATE SEQUENCE "user".notes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    MAXVALUE 2147483647
    CACHE 1;


ALTER TABLE "user".notes_id_seq OWNER TO postgres;

--
-- Name: notes_id_seq; Type: SEQUENCE OWNED BY; Schema: user; Owner: postgres
--

ALTER SEQUENCE "user".notes_id_seq OWNED BY "user".notes.id;


--
-- Name: address id; Type: DEFAULT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".address ALTER COLUMN id SET DEFAULT nextval('"user".address_id_seq'::regclass);


--
-- Name: base id; Type: DEFAULT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".base ALTER COLUMN id SET DEFAULT nextval('"user".base_id_seq'::regclass);


--
-- Name: login_records id; Type: DEFAULT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".login_records ALTER COLUMN id SET DEFAULT nextval('"user".login_records_id_seq'::regclass);


--
-- Name: notes id; Type: DEFAULT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".notes ALTER COLUMN id SET DEFAULT nextval('"user".notes_id_seq'::regclass);


--
-- Data for Name: address; Type: TABLE DATA; Schema: user; Owner: postgres
--

COPY "user".address (id, user_id, ip, version, time_offset, location, path, adapters, adapters_hash, uninstall_id, disk_id, create_time) FROM stdin;
\.


--
-- Data for Name: base; Type: TABLE DATA; Schema: user; Owner: postgres
--

COPY "user".base (id, name, name_safe, password, email, privileges, country, create_time, update_time) FROM stdin;
\.


--
-- Data for Name: friends; Type: TABLE DATA; Schema: user; Owner: postgres
--

COPY "user".friends (user_id, friend_id, remark, create_time) FROM stdin;
\.


--
-- Data for Name: login_records; Type: TABLE DATA; Schema: user; Owner: postgres
--

COPY "user".login_records (id, user_id, address_id, count, timestamps, create_time, update_time) FROM stdin;
\.


--
-- Data for Name: notes; Type: TABLE DATA; Schema: user; Owner: postgres
--

COPY "user".notes (id, user_id, note, type, added_by, create_time, update_time) FROM stdin;
\.


--
-- Name: address_id_seq; Type: SEQUENCE SET; Schema: user; Owner: postgres
--

SELECT pg_catalog.setval('"user".address_id_seq', 1, false);


--
-- Name: base_id_seq; Type: SEQUENCE SET; Schema: user; Owner: postgres
--

SELECT pg_catalog.setval('"user".base_id_seq', 1000, false);


--
-- Name: login_records_id_seq; Type: SEQUENCE SET; Schema: user; Owner: postgres
--

SELECT pg_catalog.setval('"user".login_records_id_seq', 1, false);


--
-- Name: notes_id_seq; Type: SEQUENCE SET; Schema: user; Owner: postgres
--

SELECT pg_catalog.setval('"user".notes_id_seq', 1, false);


--
-- Name: address Address.id; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT "Address.id" UNIQUE (id);


--
-- Name: login_records Login_records.id; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT "Login_records.id" UNIQUE (id);


--
-- Name: CONSTRAINT "Login_records.id" ON login_records; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "Login_records.id" ON "user".login_records IS 'Login_records.id';


--
-- Name: notes Note.id; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "Note.id" UNIQUE (id);


--
-- Name: CONSTRAINT "Note.id" ON notes; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "Note.id" ON "user".notes IS 'note id should be unique';


--
-- Name: base Unique - some user info; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT "Unique - some user info" UNIQUE (id, name, name_safe, email);


--
-- Name: CONSTRAINT "Unique - some user info" ON base; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "Unique - some user info" ON "user".base IS 'id, name, name_safe, email should be unique';


--
-- Name: address address_pkey; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT address_pkey PRIMARY KEY (id, user_id);


--
-- Name: base base_pkey; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".base
    ADD CONSTRAINT base_pkey PRIMARY KEY (id);


--
-- Name: friends friends_pkey; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT friends_pkey PRIMARY KEY (user_id, friend_id);


--
-- Name: login_records login_records_pkey; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT login_records_pkey PRIMARY KEY (id, user_id, address_id);


--
-- Name: notes notes_pkey1; Type: CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT notes_pkey1 PRIMARY KEY (id, user_id);


--
-- Name: User.name; Type: INDEX; Schema: user; Owner: postgres
--

CREATE UNIQUE INDEX "User.name" ON "user".base USING btree (name, name_safe);


--
-- Name: base update_time_auto; Type: TRIGGER; Schema: user; Owner: postgres
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".base FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON base; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON TRIGGER update_time_auto ON "user".base IS 'auto update the update_time after update user info';


--
-- Name: login_records update_time_auto; Type: TRIGGER; Schema: user; Owner: postgres
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".login_records FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON login_records; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON TRIGGER update_time_auto ON "user".login_records IS 'auto update the update_time after update user info';


--
-- Name: notes update_time_auto; Type: TRIGGER; Schema: user; Owner: postgres
--

CREATE TRIGGER update_time_auto BEFORE UPDATE ON "user".notes FOR EACH ROW EXECUTE PROCEDURE public.update_timestamp();


--
-- Name: TRIGGER update_time_auto ON notes; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON TRIGGER update_time_auto ON "user".notes IS 'auto update the update_time after update note info';


--
-- Name: login_records Address.id; Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT "Address.id" FOREIGN KEY (address_id) REFERENCES "user".address(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "Address.id" ON login_records; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "Address.id" ON "user".login_records IS 'address_id';


--
-- Name: address User.id; Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".address
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: friends User.id; Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id" ON friends; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "User.id" ON "user".friends IS 'user_id';


--
-- Name: notes User.id; Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: login_records User.id; Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".login_records
    ADD CONSTRAINT "User.id" FOREIGN KEY (user_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id" ON login_records; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "User.id" ON "user".login_records IS 'user_id';


--
-- Name: notes User.id (added_by); Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".notes
    ADD CONSTRAINT "User.id (added_by)" FOREIGN KEY (added_by) REFERENCES "user".base(id) ON UPDATE CASCADE;


--
-- Name: friends User.id (friend); Type: FK CONSTRAINT; Schema: user; Owner: postgres
--

ALTER TABLE ONLY "user".friends
    ADD CONSTRAINT "User.id (friend)" FOREIGN KEY (friend_id) REFERENCES "user".base(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: CONSTRAINT "User.id (friend)" ON friends; Type: COMMENT; Schema: user; Owner: postgres
--

COMMENT ON CONSTRAINT "User.id (friend)" ON "user".friends IS 'user_id (friend)';


--
-- PostgreSQL database dump complete
--

