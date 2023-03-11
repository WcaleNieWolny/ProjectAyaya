#define log_info(str)                                                         \
  fprintf (stdout, "[C INFO %s, %d] %s\n", __FILE__, __LINE__, str);          \
  fflush (stdout);

#define log_error(str)                                                        \
  fprintf (stderr, "[C ERR %s, %d] %s\n", __FILE__, __LINE__, str);           \
  fflush (stderr);
