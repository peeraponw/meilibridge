FROM postgres:16

# Install pg_cron extension
RUN apt-get update && \
    apt-get install -y postgresql-16-cron && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Ensure pg_cron is loaded
RUN echo "shared_preload_libraries = 'pg_cron,pgoutput'" >> /usr/share/postgresql/postgresql.conf.sample
RUN echo "cron.database_name = 'demo'" >> /usr/share/postgresql/postgresql.conf.sample